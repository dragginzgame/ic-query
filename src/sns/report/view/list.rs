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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sns_name_sort_preserves_stable_id_tiebreaker() {
        let mut instances = vec![
            mainnet_sns(3, "beta"),
            mainnet_sns(1, "Alpha"),
            mainnet_sns(2, "alpha"),
        ];

        sort_mainnet_sns_instances(&mut instances, SnsListSort::Name);

        assert_eq!(ids(&instances), vec![1, 2, 3]);
    }

    fn ids(instances: &[MainnetSns]) -> Vec<usize> {
        instances.iter().map(|sns| sns.id).collect()
    }

    fn mainnet_sns(id: usize, name: &str) -> MainnetSns {
        MainnetSns {
            id,
            name: name.to_string(),
            description: None,
            url: None,
            root_canister_id: format!("{id}-root"),
            governance_canister_id: format!("{id}-governance"),
            ledger_canister_id: format!("{id}-ledger"),
            swap_canister_id: format!("{id}-swap"),
            index_canister_id: format!("{id}-index"),
            metadata_error: None,
        }
    }
}
