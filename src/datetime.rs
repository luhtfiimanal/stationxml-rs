//! Shared datetime parsing and formatting utilities.
//!
//! Used by both FDSN and SC3ML backends for ISO 8601 datetime handling.

use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};

use crate::error::{Result, StationXmlError};

/// Parse ISO 8601 datetime string to chrono DateTime<Utc>.
///
/// Handles multiple variants commonly found in station metadata XML:
/// - `2026-02-20T00:00:00Z` (RFC3339 with Z)
/// - `2026-02-20T00:00:00.000Z` (with fractional seconds)
/// - `2026-02-20T00:00:00+00:00` (with offset)
/// - `2026-02-20T00:00:00` (no timezone — assume UTC)
/// - `2026-02-20T00:00:00.0000Z` (microsecond precision)
pub fn parse_datetime(s: &str) -> Result<DateTime<Utc>> {
    // Try RFC3339 first (with timezone info)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    // Try without timezone (assume UTC)
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(naive.and_utc());
    }
    // Try with fractional seconds, no timezone
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
        return Ok(naive.and_utc());
    }
    Err(StationXmlError::InvalidData(format!(
        "cannot parse datetime: '{s}'"
    )))
}

/// Parse an optional datetime string.
pub fn parse_datetime_opt(s: &Option<String>) -> Result<Option<DateTime<Utc>>> {
    match s {
        Some(s) if !s.is_empty() => Ok(Some(parse_datetime(s)?)),
        _ => Ok(None),
    }
}

/// Format a DateTime<Utc> to RFC3339 with second precision.
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Format an optional DateTime<Utc>.
pub fn format_datetime_opt(dt: &Option<DateTime<Utc>>) -> Option<String> {
    dt.as_ref().map(format_datetime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn parse_rfc3339_z() {
        let dt = parse_datetime("2026-02-20T00:00:00Z").unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 2);
    }

    #[test]
    fn parse_rfc3339_offset() {
        let dt = parse_datetime("2026-02-20T00:00:00+00:00").unwrap();
        assert_eq!(dt.year(), 2026);
    }

    #[test]
    fn parse_no_timezone() {
        let dt = parse_datetime("2026-02-20T00:00:00").unwrap();
        assert_eq!(dt.year(), 2026);
    }

    #[test]
    fn parse_fractional() {
        let dt = parse_datetime("2026-02-20T12:30:45.123Z").unwrap();
        assert_eq!(dt.hour(), 12);
    }

    #[test]
    fn parse_fractional_no_tz() {
        let dt = parse_datetime("2026-02-20T12:30:45.123").unwrap();
        assert_eq!(dt.hour(), 12);
    }

    #[test]
    fn parse_microseconds() {
        let dt = parse_datetime("2026-02-20T12:30:45.0000Z").unwrap();
        assert_eq!(dt.hour(), 12);
    }

    #[test]
    fn parse_invalid() {
        assert!(parse_datetime("not-a-date").is_err());
    }

    #[test]
    fn parse_opt_none() {
        assert_eq!(parse_datetime_opt(&None).unwrap(), None);
    }

    #[test]
    fn parse_opt_empty() {
        assert_eq!(parse_datetime_opt(&Some("".into())).unwrap(), None);
    }

    #[test]
    fn format_roundtrip() {
        let dt = parse_datetime("2026-02-20T12:30:45Z").unwrap();
        let s = format_datetime(&dt);
        assert_eq!(s, "2026-02-20T12:30:45Z");
    }

    #[test]
    fn format_opt_none() {
        assert_eq!(format_datetime_opt(&None), None);
    }
}
