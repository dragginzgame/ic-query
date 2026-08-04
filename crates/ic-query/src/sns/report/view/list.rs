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
        instances.retain(|sns| sns.lifecycle == Some(COMMITTED_SNS_LIFECYCLE));
    }
}

/// Sort deployed SNS instances for list reports while preserving assigned ids.
pub(in crate::sns::report) fn sort_mainnet_sns_instances(
    instances: &mut [MainnetSns],
    sort: SnsListSort,
) {
    match sort {
        SnsListSort::Id => sort_mainnet_sns_instances_by_id(instances),
        SnsListSort::Name => instances.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.cmp(&right.id))
        }),
    }
}

fn sort_mainnet_sns_instances_by_id(instances: &mut [MainnetSns]) {
    instances.sort_by_key(|sns| sns.id);
}
