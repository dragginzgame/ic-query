use super::{fixtures::*, *};

#[test]
fn stale_status_is_deterministic() {
    let catalog = fixture_catalog();
    let fresh = catalog_stale_status(&catalog, 1_780_531_300, 200);
    let stale = catalog_stale_status(&catalog, 1_780_531_501, 200);

    assert!(!fresh.catalog_stale);
    assert!(stale.catalog_stale);
}

#[test]
fn stale_duration_accepts_units() {
    assert_eq!(parse_stale_after_duration("7d").expect("days"), 604_800);
    assert_eq!(parse_stale_after_duration("2h").expect("hours"), 7_200);
    assert_eq!(parse_stale_after_duration("30m").expect("minutes"), 1_800);
    assert_eq!(parse_stale_after_duration("90s").expect("seconds"), 90);
    assert_eq!(parse_stale_after_duration("42").expect("bare"), 42);
    assert!(matches!(
        parse_stale_after_duration("0d"),
        Err(SubnetCatalogHostError::InvalidStaleDuration { .. })
    ));
}

#[test]
fn utc_timestamp_formatter_is_deterministic() {
    assert_eq!(format_utc_timestamp_secs(0), "1970-01-01T00:00:00Z");
    assert_eq!(
        format_utc_timestamp_secs(1_780_531_200),
        "2026-06-04T00:00:00Z"
    );
}
