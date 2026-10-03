//! Module: subnet_catalog::time
//!
//! Responsibility: format catalog timestamps and derive cache staleness metadata.
//!
//! Does not own: cache refresh policy, host filesystem paths, or report rendering.
//!
//! Boundary: keeps timestamp parsing and display deterministic without introducing
//! live clock reads into report builders or cache loaders.

#[cfg(feature = "subnet-catalog-host")]
use super::{CatalogStaleStatus, RawSubnetCatalog};
#[cfg(feature = "subnet-catalog-host")]
use crate::freshness::freshness_facts;

/// Computes stale/fresh metadata for a catalog relative to a caller-provided time.
#[must_use]
#[cfg(feature = "subnet-catalog-host")]
pub fn catalog_stale_status(
    catalog: &RawSubnetCatalog,
    now_unix_secs: u64,
    stale_after_seconds: u64,
) -> CatalogStaleStatus {
    let freshness = freshness_facts(
        parse_utc_timestamp_secs(&catalog.provenance.fetched_at),
        now_unix_secs,
        stale_after_seconds,
    );
    CatalogStaleStatus {
        catalog_stale: freshness.stale,
        stale_reason: freshness.reason.to_string(),
        stale_after_seconds: freshness.stale_after_seconds,
        fetched_at_unix_secs: freshness.fetched_at_unix_secs,
        age_seconds: freshness.age_seconds,
    }
}

/// Parse the canonical UTC representation emitted by `format_utc_timestamp_secs`.
pub fn parse_utc_timestamp_secs(value: &str) -> Option<u64> {
    let original = value;
    let value = value.strip_suffix('Z')?;
    let (date, time) = value.split_once('T')?;
    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse::<i64>().ok()?;
    let month = date_parts.next()?.parse::<u32>().ok()?;
    let day = date_parts.next()?.parse::<u32>().ok()?;
    if date_parts.next().is_some() {
        return None;
    }
    let mut time_parts = time.split(':');
    let hour = time_parts.next()?.parse::<u32>().ok()?;
    let minute = time_parts.next()?.parse::<u32>().ok()?;
    let second = time_parts.next()?.parse::<u32>().ok()?;
    if time_parts.next().is_some()
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let days = days_from_civil(year, month, day)?;
    let seconds = days
        .checked_mul(86_400)?
        .checked_add(i64::from(hour) * 3_600)?
        .checked_add(i64::from(minute) * 60)?
        .checked_add(i64::from(second))?;
    let seconds = u64::try_from(seconds).ok()?;
    (format_utc_timestamp_secs(seconds) == original).then_some(seconds)
}

/// Formats a Unix timestamp as a UTC RFC3339-like timestamp with second precision.
pub fn format_utc_timestamp_secs(value: u64) -> String {
    let days = i64::try_from(value / 86_400).unwrap_or(i64::MAX);
    let seconds_of_day = value % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> Option<i64> {
    let month = i64::from(month);
    let day = i64::from(day);
    let year = year.checked_sub(i64::from(month <= 2))?;
    let era = if year >= 0 {
        year
    } else {
        year.checked_sub(399)?
    } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era.checked_mul(146_097)?
        .checked_add(day_of_era)?
        .checked_sub(719_468)
}

#[cfg(test)]
mod timestamp_tests {
    use super::{format_utc_timestamp_secs, parse_utc_timestamp_secs};

    #[test]
    fn timestamp_parser_accepts_canonical_formatter_output() {
        for seconds in [
            0,
            1,
            86_400,
            951_782_400,
            1_780_531_200,
            253_402_300_800,
            i64::MAX.unsigned_abs(),
        ] {
            let timestamp = format_utc_timestamp_secs(seconds);
            assert_eq!(parse_utc_timestamp_secs(&timestamp), Some(seconds));
        }
    }

    #[test]
    fn timestamp_parser_requires_canonical_utc_text() {
        for timestamp in [
            "2026-6-04T00:00:00Z",
            "2026-06-4T00:00:00Z",
            "2026-06-04T0:00:00Z",
            "2026-06-04T00:0:00Z",
            "2026-06-04T00:00:0Z",
            "002026-06-04T00:00:00Z",
            "+2026-06-04T00:00:00Z",
            "2026-+06-04T00:00:00Z",
            "2026-06-04T00:00:000Z",
            "2026-06-04T00:00:00.000Z",
            "2026-06-04T00:00:00+00:00",
            "2026-06-04t00:00:00z",
            " 2026-06-04T00:00:00Z",
            "2026-06-04T00:00:00Z ",
        ] {
            assert_eq!(parse_utc_timestamp_secs(timestamp), None, "{timestamp:?}");
        }
    }

    #[test]
    fn timestamp_parser_rejects_impossible_calendar_dates() {
        assert_eq!(parse_utc_timestamp_secs("2026-02-29T00:00:00Z"), None);
        assert_eq!(parse_utc_timestamp_secs("2026-04-31T00:00:00Z"), None);
        assert!(parse_utc_timestamp_secs("2028-02-29T00:00:00Z").is_some());
    }

    #[test]
    fn timestamp_parser_rejects_out_of_range_years_without_panicking() {
        for year in [i64::MIN, i64::MAX, 25_252_734_927_766_555] {
            assert_eq!(
                parse_utc_timestamp_secs(&format!("{year}-01-01T00:00:00Z")),
                None
            );
        }
    }
}
