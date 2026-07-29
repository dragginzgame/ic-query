//! Module: freshness
//!
//! Responsibility: derive caller-relative freshness facts from parsed timestamps.
//! Does not own: timestamp parsing, cache refresh policy, or public report models.
//! Boundary: keeps stale/future/unparseable decisions consistent across caches.

///
/// FreshnessFacts
///
/// Internal caller-relative freshness result shared by cache report families.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FreshnessFacts {
    pub(crate) stale: bool,
    pub(crate) reason: &'static str,
    pub(crate) stale_after_seconds: u64,
    pub(crate) fetched_at_unix_secs: Option<u64>,
    pub(crate) age_seconds: Option<u64>,
}

pub const fn freshness_facts(
    fetched_at_unix_secs: Option<u64>,
    now_unix_secs: u64,
    stale_after_seconds: u64,
) -> FreshnessFacts {
    let Some(fetched_at_unix_secs) = fetched_at_unix_secs else {
        return FreshnessFacts {
            stale: true,
            reason: "fetched_at_unparseable",
            stale_after_seconds,
            fetched_at_unix_secs: None,
            age_seconds: None,
        };
    };
    let Some(age_seconds) = now_unix_secs.checked_sub(fetched_at_unix_secs) else {
        return FreshnessFacts {
            stale: true,
            reason: "fetched_at_in_future",
            stale_after_seconds,
            fetched_at_unix_secs: Some(fetched_at_unix_secs),
            age_seconds: None,
        };
    };
    let stale = age_seconds > stale_after_seconds;
    FreshnessFacts {
        stale,
        reason: if stale { "expired" } else { "fresh" },
        stale_after_seconds,
        fetched_at_unix_secs: Some(fetched_at_unix_secs),
        age_seconds: Some(age_seconds),
    }
}

#[cfg(test)]
mod tests {
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
}
