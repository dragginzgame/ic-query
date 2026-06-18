//! Module: sns::report::cache_summary
//!
//! Responsibility: share cache-summary view helpers across SNS cache reports.
//! Does not own: cache storage, refresh attempts, or text rendering.
//! Boundary: keeps common cache-summary ordering deterministic.

use crate::sns::report::SnsHostError;
use candid::Principal;

pub(in crate::sns::report) trait SnsCacheSummarySortKey {
    fn id(&self) -> usize;
    fn root_canister_id(&self) -> &str;
}

pub(in crate::sns::report) fn parse_sns_root_canister_input(
    input: &str,
) -> Result<String, SnsHostError> {
    Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })
        .map(|principal| principal.to_text())
}

pub(in crate::sns::report) fn sort_sns_cache_summaries<T>(caches: &mut [T])
where
    T: SnsCacheSummarySortKey,
{
    caches.sort_by(|left, right| {
        left.id()
            .cmp(&right.id())
            .then_with(|| left.root_canister_id().cmp(right.root_canister_id()))
    });
}
