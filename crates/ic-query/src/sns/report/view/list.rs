//! Module: sns::report::view::list
//!
//! Responsibility: apply deployed SNS list view ordering.
//! Does not own: SNS-W fetching, lookup id assignment, report assembly, or rendering.
//! Boundary: sorts deployed SNS rows while preserving already-assigned stable ids.

use crate::sns::report::{SnsListSort, source::MainnetSns};

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
