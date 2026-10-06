//! IDs and timestamps used by every table.
//!
//! Spec hard rules: IDs are UUID v7 strings. Times are stored as UTC ISO 8601.
//! The user sees times in America/Anchorage.

use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use chrono_tz::Tz;

/// The time zone the user sees. Stored data is always UTC.
pub const USER_TZ: Tz = chrono_tz::America::Anchorage;

/// A new UUID v7 as a string. v7 IDs sort by creation time.
pub fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

/// Current time as a UTC ISO 8601 string, for `created_at` and `updated_at`.
pub fn now_iso() -> String {
    to_iso(Utc::now())
}

/// Formats a UTC time the way we store it, for example `2026-10-06T04:27:35Z`.
pub fn to_iso(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Today's date on the user's wall clock (Anchorage), not UTC.
pub fn local_today(now: DateTime<Utc>) -> NaiveDate {
    now.with_timezone(&USER_TZ).date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn ids_are_v7_and_sortable() {
        let a = new_id();
        let b = new_id();
        assert_eq!(uuid::Uuid::parse_str(&a).unwrap().get_version_num(), 7);
        assert!(a < b);
    }

    #[test]
    fn local_today_uses_anchorage() {
        // 05:00 UTC on Oct 6 is still Oct 5 in Anchorage (UTC-8 in October).
        let t = Utc.with_ymd_and_hms(2026, 10, 6, 5, 0, 0).unwrap();
        assert_eq!(local_today(t), NaiveDate::from_ymd_opt(2026, 10, 5).unwrap());
    }
}
