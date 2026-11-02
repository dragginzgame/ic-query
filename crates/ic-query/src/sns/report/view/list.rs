//! Module: sns::report::view::list
//!
//! Responsibility: apply deployed SNS list lifecycle filtering and ordering.
//! Does not own: SNS-W fetching, lookup id assignment, report assembly, or rendering.
//! Boundary: sorts deployed SNS rows while preserving already-assigned stable ids.

use crate::sns::report::{SnsListSort, source::MainnetSns};

const COMMITTED_SNS_LIFECYCLE: i32 = 3;

/// Retain either every catalog row or only successfully committed SNS launches.
pub(in crate::sns::report) fn filter_mainnet_sns_instances(
    instances: &mut Vec<MainnetSns>,
    all_lifecycles: bool,
) {
    if !all_lifecycles {
        instances.retain(|sns| sns.lifecycle.value() == Some(COMMITTED_SNS_LIFECYCLE));
    }
}

/// Preserve admitted ascending IDs or sort by name; lifecycle filtering retains ID order.
pub(in crate::sns::report) fn sort_mainnet_sns_instances(
    instances: &mut [MainnetSns],
    sort: SnsListSort,
) {
    match sort {
        SnsListSort::Id => {}
        SnsListSort::Name => instances.sort_by_cached_key(|sns| (sns.name.to_lowercase(), sns.id)),
    }
}
