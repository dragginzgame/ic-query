use super::{FreshnessFacts, freshness_facts};

#[test]
fn freshness_classifies_missing_future_fresh_and_expired_timestamps() {
    let cases = [
        (
            None,
            FreshnessFacts {
                stale: true,
                reason: "fetched_at_unparseable",
                stale_after_seconds: 10,
                fetched_at_unix_secs: None,
                age_seconds: None,
            },
        ),
        (
            Some(101),
            FreshnessFacts {
                stale: true,
                reason: "fetched_at_in_future",
                stale_after_seconds: 10,
                fetched_at_unix_secs: Some(101),
                age_seconds: None,
            },
        ),
        (
            Some(90),
            FreshnessFacts {
                stale: false,
                reason: "fresh",
                stale_after_seconds: 10,
                fetched_at_unix_secs: Some(90),
                age_seconds: Some(10),
            },
        ),
        (
            Some(89),
            FreshnessFacts {
                stale: true,
                reason: "expired",
                stale_after_seconds: 10,
                fetched_at_unix_secs: Some(89),
                age_seconds: Some(11),
            },
        ),
    ];

    for (fetched_at, expected) in cases {
        assert_eq!(freshness_facts(fetched_at, 100, 10), expected);
    }
}
