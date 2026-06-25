//! Module: sns::report::lookup::ids
//!
//! Responsibility: assign stable deployed SNS list ids from SNS-W order.
//! Does not own: list sorting, source fetching, lookup parsing, or rendering.
//! Boundary: mutates source rows before lookup or view sorting occurs.

use crate::sns::report::source::MainnetSns;

/// Assign stable one-based ids in the current SNS-W order.
pub(in crate::sns::report) fn assign_sns_ids_in_current_order(instances: &mut [MainnetSns]) {
    for (index, sns) in instances.iter_mut().enumerate() {
        sns.id = index + 1;
    }
}

pub(in crate::sns::report::lookup) fn sort_sns_by_assigned_id(instances: &mut [MainnetSns]) {
    instances.sort_by_key(|sns| sns.id);
}
