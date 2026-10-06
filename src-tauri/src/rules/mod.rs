//! Rules: "when X happens, do Y".
//!
//! This step defines the JSON rule format from the spec and the timing
//! for `time` and `offset` triggers. The agent that runs rules every 60
//! seconds and the action runner come in the next Phase 1 step.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::db::ids::USER_TZ;
use crate::scheduler;

/// Time of day used when a trigger only gives a date. A default, see docs/DECISIONS.md.
pub const DEFAULT_FIRE_TIME: (u32, u32) = (9, 0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub name: String,
    pub trigger: Trigger,
    #[serde(default)]
    pub conditions: Vec<serde_json::Value>,
    pub actions: Vec<Action>,
    #[serde(default = "default_ladder")]
    pub max_ladder: u8,
}

fn default_ladder() -> u8 {
    2
}

/// Trigger types from the spec's table. Field names match the spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    Time {
        #[serde(default)]
        at: Option<String>,
        #[serde(default)]
        rrule: Option<String>,
    },
    Offset {
        #[serde(default)]
        event: Option<String>,
        #[serde(default)]
        date_field: Option<String>,
        days_before: i64,
    },
    Meter {
        item: String,
        unit: String,
        #[serde(default)]
        every: Option<f64>,
        #[serde(default)]
        at_value: Option<f64>,
    },
    DataChange {
        table: String,
        #[serde(default)]
        r#match: serde_json::Value,
    },
    Weather {
        condition: String,
        #[serde(default)]
        threshold: Option<f64>,
    },
    Inbox {
        kind: String,
    },
    Pattern {
        item_type: String,
        absent_days: i64,
    },
}

/// Action types from the spec. Extra fields are kept as-is for the runner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Notify(#[serde(default)] serde_json::Map<String, serde_json::Value>),
    CreateTask(#[serde(default)] serde_json::Map<String, serde_json::Value>),
    AddEvent(#[serde(default)] serde_json::Map<String, serde_json::Value>),
    AddToList(#[serde(default)] serde_json::Map<String, serde_json::Value>),
    StartChecklist(#[serde(default)] serde_json::Map<String, serde_json::Value>),
    Log(#[serde(default)] serde_json::Map<String, serde_json::Value>),
    SuggestSlot(#[serde(default)] serde_json::Map<String, serde_json::Value>),
    Ask(#[serde(default)] serde_json::Map<String, serde_json::Value>),
}

pub fn parse(json: &str) -> Result<Rule, String> {
    let rule: Rule = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if rule.actions.is_empty() {
        return Err("A rule needs at least one action.".into());
    }
    if !(1..=4).contains(&rule.max_ladder) {
        return Err("max_ladder must be 1 to 4.".into());
    }
    if let Trigger::Time { at: None, rrule: None } = rule.trigger {
        return Err("A time trigger needs `at` or `rrule`.".into());
    }
    Ok(rule)
}

/// Next fire time for a `time` trigger strictly after `after`.
/// `dtstart` anchors repeats (usually the rule's creation time).
pub fn next_time_fire(trigger: &Trigger, dtstart: DateTime<Utc>, after: DateTime<Utc>) -> Result<Option<DateTime<Utc>>, String> {
    match trigger {
        Trigger::Time { rrule: Some(r), .. } => scheduler::next_after(r, dtstart, after),
        Trigger::Time { at: Some(at), .. } => {
            let t = DateTime::parse_from_rfc3339(at).map_err(|e| e.to_string())?.with_timezone(&Utc);
            Ok((t > after).then_some(t))
        }
        _ => Err("Not a time trigger.".into()),
    }
}

/// Fire time for an `offset` trigger, given the target date it counts back from.
/// Example: passport expires 2027-07-01, 270 days before, fires 9 AM local on that day.
pub fn offset_fire(target: NaiveDate, days_before: i64) -> DateTime<Utc> {
    let day = target - Duration::days(days_before);
    let time = NaiveTime::from_hms_opt(DEFAULT_FIRE_TIME.0, DEFAULT_FIRE_TIME.1, 0).unwrap();
    USER_TZ.from_local_datetime(&day.and_time(time)).earliest().unwrap().with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spec_example() {
        // The example from docs/SPEC.md, "Rule format".
        let r = parse(
            r#"{
              "name": "Oil change by mileage",
              "trigger": { "type": "meter", "item": "<vehicle item id>", "unit": "mi", "every": 5000 },
              "conditions": [],
              "actions": [
                { "type": "create_task", "title": "Oil change", "energy": "medium", "est_minutes": 60 },
                { "type": "suggest_slot", "within_days": 7 }
              ],
              "max_ladder": 3
            }"#,
        )
        .unwrap();
        assert!(matches!(r.trigger, Trigger::Meter { every: Some(e), .. } if e == 5000.0));
        assert_eq!(r.actions.len(), 2);
        match &r.actions[0] {
            Action::CreateTask(m) => assert_eq!(m["title"], "Oil change"),
            other => panic!("wrong action {other:?}"),
        }
    }

    #[test]
    fn rejects_bad_rules() {
        assert!(parse(r#"{"name":"x","trigger":{"type":"time"},"actions":[{"type":"notify"}]}"#).is_err());
        assert!(parse(r#"{"name":"x","trigger":{"type":"time","at":"2026-10-06T00:00:00Z"},"actions":[]}"#).is_err());
        assert!(parse(r#"{"name":"x","trigger":{"type":"bogus"},"actions":[{"type":"notify"}]}"#).is_err());
        assert!(parse(r#"{"name":"x","trigger":{"type":"time","at":"2026-10-06T00:00:00Z"},"actions":[{"type":"notify"}],"max_ladder":9}"#).is_err());
    }

    #[test]
    fn default_ladder_is_2() {
        let r = parse(r#"{"name":"x","trigger":{"type":"time","rrule":"FREQ=DAILY"},"actions":[{"type":"notify","text":"hi"}]}"#).unwrap();
        assert_eq!(r.max_ladder, 2);
    }

    #[test]
    fn time_trigger_at_and_rrule() {
        let start = Utc.with_ymd_and_hms(2026, 10, 1, 17, 0, 0).unwrap(); // 9 AM local
        let at = Trigger::Time { at: Some("2026-10-10T17:00:00Z".into()), rrule: None };
        assert_eq!(next_time_fire(&at, start, start).unwrap(), Some(Utc.with_ymd_and_hms(2026, 10, 10, 17, 0, 0).unwrap()));
        assert_eq!(next_time_fire(&at, start, Utc.with_ymd_and_hms(2026, 10, 11, 0, 0, 0).unwrap()).unwrap(), None);
        // Furnace filter every 90 days.
        let rr = Trigger::Time { at: None, rrule: Some("FREQ=DAILY;INTERVAL=90".into()) };
        assert_eq!(next_time_fire(&rr, start, start).unwrap(), Some(Utc.with_ymd_and_hms(2026, 12, 30, 18, 0, 0).unwrap()));
    }

    #[test]
    fn offset_counts_back_in_local_time() {
        let fire = offset_fire(NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(), 5);
        // Oct 10 at 9 AM Anchorage (UTC-8) is 17:00 UTC.
        assert_eq!(fire, Utc.with_ymd_and_hms(2026, 10, 10, 17, 0, 0).unwrap());
    }
}
