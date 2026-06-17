//! Module: sns::report::lookup::sort
//!
//! Responsibility: deployed SNS list id assignment and sorting.
//! Does not own: SNS-W fetching, lookup input parsing, or report rendering.
//! Boundary: preserves stable ids while applying report-level list ordering.

use crate::sns::report::{SnsListSort, source::MainnetSns};

/// Assign stable one-based ids in the current SNS-W order.
pub(in crate::sns::report) fn assign_sns_ids_in_current_order(instances: &mut [MainnetSns]) {
    for (index, sns) in instances.iter_mut().enumerate() {
        sns.id = index + 1;
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
