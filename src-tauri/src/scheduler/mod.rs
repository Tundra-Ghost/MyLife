//! Reminder timing: repeats, quiet hours, the reminder ladder, and
//! catching up after sleep.
//!
//! These are pure functions so they are easy to test. The tray agent
//! (next Phase 1 step) calls them every 60 seconds.

use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use rrule::{RRuleSet, Tz as RTz};

use crate::db::ids::USER_TZ;

/// Spec default quiet hours: 10 PM to 7 AM local.
pub const QUIET_START: (u32, u32) = (22, 0);
pub const QUIET_END: (u32, u32) = (7, 0);

/// Ladder timing from the spec.
pub const LEVEL3_AFTER: Duration = Duration::hours(2);
pub const LEVEL3_REPEAT: Duration = Duration::hours(4);

/// Builds an RRULE set anchored at `dtstart`, read in the user's time zone.
/// Using local time means "9 AM every month on the 31st" stays 9 AM
/// across daylight saving changes, and month-end follows RFC 5545.
fn rule_set(rrule: &str, dtstart: DateTime<Utc>) -> Result<RRuleSet, String> {
    let local = dtstart.with_timezone(&USER_TZ);
    let text = format!(
        "DTSTART;TZID={}:{}\nRRULE:{}",
        USER_TZ.name(),
        local.format("%Y%m%dT%H%M%S"),
        rrule.trim_start_matches("RRULE:")
    );
    text.parse::<RRuleSet>().map_err(|e| e.to_string())
}

/// The first occurrence strictly after `after`, or `None` if the rule has ended.
pub fn next_after(rrule: &str, dtstart: DateTime<Utc>, after: DateTime<Utc>) -> Result<Option<DateTime<Utc>>, String> {
    let set = rule_set(rrule, dtstart)?.after(after.with_timezone(&RTz::UTC));
    // `after` in the rrule crate is inclusive, so skip an exact match.
    Ok(set
        .all(2)
        .dates
        .into_iter()
        .map(|d| d.with_timezone(&Utc))
        .find(|d| *d > after))
}

/// After sleep or shutdown: the most recent occurrence in `(last_check, now]`.
/// Spec: each missed reminder fires once on wake. Never fire a backlog.
pub fn missed_once(
    rrule: &str,
    dtstart: DateTime<Utc>,
    last_check: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, String> {
    let set = rule_set(rrule, dtstart)?
        .after(last_check.with_timezone(&RTz::UTC))
        .before(now.with_timezone(&RTz::UTC));
    // Cap the scan so a yearly-old minutely rule can't hang the agent.
    Ok(set
        .all(10_000)
        .dates
        .into_iter()
        .map(|d| d.with_timezone(&Utc))
        .rfind(|d| *d > last_check && *d <= now))
}

/// True when `t` falls inside quiet hours on the user's clock.
pub fn in_quiet_hours(t: DateTime<Utc>) -> bool {
    let local = t.with_timezone(&USER_TZ).time();
    let start = NaiveTime::from_hms_opt(QUIET_START.0, QUIET_START.1, 0).unwrap();
    let end = NaiveTime::from_hms_opt(QUIET_END.0, QUIET_END.1, 0).unwrap();
    // The window wraps past midnight.
    local >= start || local < end
}

/// When a reminder may actually show. Urgent reminders (bills due today)
/// break through quiet hours. Others wait until quiet hours end.
pub fn deliver_at(fire_at: DateTime<Utc>, urgent: bool) -> DateTime<Utc> {
    if urgent || !in_quiet_hours(fire_at) {
        return fire_at;
    }
    let local = fire_at.with_timezone(&USER_TZ);
    let end = NaiveTime::from_hms_opt(QUIET_END.0, QUIET_END.1, 0).unwrap();
    let mut day = local.date_naive();
    if local.time() >= end {
        // Evening part of the window: quiet ends tomorrow morning.
        day = day.succ_opt().unwrap();
    }
    USER_TZ
        .from_local_datetime(&day.and_time(end))
        .earliest()
        .unwrap()
        .with_timezone(&Utc)
}

/// The ladder level a reminder should be at right now.
///
/// 1 = quiet (Today view only), 2 = desktop nudge at `fire_at`,
/// 3 = persistent, from 2 hours after `fire_at` if not acted on.
/// Level 4 (phone push) arrives in Phase 4, so it is capped at 3 here.
pub fn ladder_level(fire_at: DateTime<Utc>, now: DateTime<Utc>, max_ladder: u8) -> u8 {
    let max = max_ladder.clamp(1, 3);
    let level = if now < fire_at {
        1
    } else if now < fire_at + LEVEL3_AFTER {
        2
    } else {
        3
    };
    level.min(max)
}

/// When a level 3 reminder should show again: every 4 hours after it starts.
pub fn next_level3_repeat(fire_at: DateTime<Utc>, now: DateTime<Utc>) -> DateTime<Utc> {
    let first = fire_at + LEVEL3_AFTER;
    if now < first {
        return first;
    }
    let done = (now - first).num_seconds() / LEVEL3_REPEAT.num_seconds() + 1;
    first + LEVEL3_REPEAT * done as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a UTC time from Anchorage wall-clock values.
    fn ak(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        USER_TZ.with_ymd_and_hms(y, m, d, h, min, 0).earliest().unwrap().with_timezone(&Utc)
    }

    #[test]
    fn daily_repeat() {
        let start = ak(2026, 10, 1, 9, 0);
        let next = next_after("FREQ=DAILY", start, ak(2026, 10, 6, 9, 0)).unwrap();
        assert_eq!(next, Some(ak(2026, 10, 7, 9, 0)));
    }

    #[test]
    fn month_end_skips_short_months() {
        // RFC 5545: BYMONTHDAY=31 skips months without a 31st.
        let start = ak(2026, 1, 31, 9, 0);
        let next = next_after("FREQ=MONTHLY;BYMONTHDAY=31", start, start).unwrap();
        assert_eq!(next, Some(ak(2026, 3, 31, 9, 0)));
    }

    #[test]
    fn last_day_of_month() {
        // BYMONTHDAY=-1 means the last day, including February.
        let start = ak(2026, 1, 31, 9, 0);
        let next = next_after("FREQ=MONTHLY;BYMONTHDAY=-1", start, start).unwrap();
        assert_eq!(next, Some(ak(2026, 2, 28, 9, 0)));
        let leap = next_after("FREQ=MONTHLY;BYMONTHDAY=-1", ak(2028, 1, 31, 9, 0), ak(2028, 1, 31, 9, 0)).unwrap();
        assert_eq!(leap, Some(ak(2028, 2, 29, 9, 0)));
    }

    #[test]
    fn repeat_keeps_local_time_across_dst() {
        // DST ends Nov 1 2026. A 9 AM daily stays at 9 AM local.
        let start = ak(2026, 10, 30, 9, 0);
        let next = next_after("FREQ=DAILY", start, ak(2026, 11, 1, 8, 0)).unwrap();
        assert_eq!(next, Some(ak(2026, 11, 1, 9, 0)));
    }

    #[test]
    fn ended_rule_has_no_next() {
        let start = ak(2026, 10, 1, 9, 0);
        assert_eq!(next_after("FREQ=DAILY;COUNT=2", start, ak(2026, 10, 5, 0, 0)).unwrap(), None);
    }

    #[test]
    fn bad_rule_is_an_error() {
        assert!(next_after("FREQ=SOMETIMES", ak(2026, 10, 1, 9, 0), ak(2026, 10, 1, 9, 0)).is_err());
    }

    #[test]
    fn missed_reminders_fire_once() {
        // PC slept for 3 days over a daily 9 AM reminder.
        let start = ak(2026, 10, 1, 9, 0);
        let fire = missed_once("FREQ=DAILY", start, ak(2026, 10, 2, 10, 0), ak(2026, 10, 5, 12, 0)).unwrap();
        assert_eq!(fire, Some(ak(2026, 10, 5, 9, 0)));
        // Nothing missed.
        let none = missed_once("FREQ=DAILY", start, ak(2026, 10, 5, 10, 0), ak(2026, 10, 5, 12, 0)).unwrap();
        assert_eq!(none, None);
    }

    #[test]
    fn quiet_hours() {
        assert!(in_quiet_hours(ak(2026, 10, 6, 23, 0)));
        assert!(in_quiet_hours(ak(2026, 10, 6, 3, 0)));
        assert!(!in_quiet_hours(ak(2026, 10, 6, 7, 0)));
        assert!(!in_quiet_hours(ak(2026, 10, 6, 21, 59)));
    }

    #[test]
    fn quiet_hours_delay_unless_urgent() {
        assert_eq!(deliver_at(ak(2026, 10, 6, 23, 0), false), ak(2026, 10, 7, 7, 0));
        assert_eq!(deliver_at(ak(2026, 10, 6, 3, 0), false), ak(2026, 10, 6, 7, 0));
        assert_eq!(deliver_at(ak(2026, 10, 6, 23, 0), true), ak(2026, 10, 6, 23, 0));
        assert_eq!(deliver_at(ak(2026, 10, 6, 12, 0), false), ak(2026, 10, 6, 12, 0));
    }

    #[test]
    fn ladder_climbs_and_caps() {
        let f = ak(2026, 10, 6, 12, 0);
        assert_eq!(ladder_level(f, f - Duration::minutes(1), 3), 1);
        assert_eq!(ladder_level(f, f, 3), 2);
        assert_eq!(ladder_level(f, f + Duration::minutes(119), 3), 2);
        assert_eq!(ladder_level(f, f + Duration::hours(2), 3), 3);
        // Rule allows only level 2 (the default).
        assert_eq!(ladder_level(f, f + Duration::hours(5), 2), 2);
        // Quiet-only rules never notify.
        assert_eq!(ladder_level(f, f + Duration::hours(5), 1), 1);
        // Level 4 is not built yet.
        assert_eq!(ladder_level(f, f + Duration::hours(50), 4), 3);
    }

    #[test]
    fn level3_repeats_every_4_hours() {
        let f = ak(2026, 10, 6, 8, 0);
        assert_eq!(next_level3_repeat(f, f), ak(2026, 10, 6, 10, 0));
        assert_eq!(next_level3_repeat(f, ak(2026, 10, 6, 10, 0)), ak(2026, 10, 6, 14, 0));
        assert_eq!(next_level3_repeat(f, ak(2026, 10, 6, 15, 0)), ak(2026, 10, 6, 18, 0));
    }
}
