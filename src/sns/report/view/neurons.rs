//! Module: sns::report::view::neurons
//!
//! Responsibility: apply SNS neuron list view ordering.
//! Does not own: neuron fetching, cache loading, report assembly, or text rendering.
//! Boundary: sorts neuron rows without changing cache identity.

use crate::sns::report::{SnsNeuronRow, SnsNeuronsSort};
use std::cmp::Reverse;

pub(in crate::sns::report) fn sort_sns_neurons(neurons: &mut [SnsNeuronRow], sort: SnsNeuronsSort) {
    match sort {
        SnsNeuronsSort::Api => {}
        SnsNeuronsSort::Id => neurons.sort_by(|left, right| left.neuron_id.cmp(&right.neuron_id)),
        SnsNeuronsSort::Stake => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.cached_neuron_stake_e8s),
                neuron.neuron_id.clone(),
            )
        }),
        SnsNeuronsSort::Maturity => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.maturity_e8s_equivalent),
                neuron.neuron_id.clone(),
            )
        }),
        SnsNeuronsSort::Created => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.created_timestamp_seconds),
                neuron.neuron_id.clone(),
            )
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neuron_stake_sort_orders_highest_stake_first_with_id_tiebreaker() {
        let mut neurons = vec![
            neuron_row("bb", 10, 1, 1),
            neuron_row("aa", 10, 2, 2),
            neuron_row("cc", 20, 1, 3),
        ];

        sort_sns_neurons(&mut neurons, SnsNeuronsSort::Stake);

        assert_eq!(neuron_ids(&neurons), vec!["cc", "aa", "bb"]);
    }

    fn neuron_ids(neurons: &[SnsNeuronRow]) -> Vec<&str> {
        neurons
            .iter()
            .map(|neuron| neuron.neuron_id.as_str())
            .collect()
    }

    fn neuron_row(
        neuron_id: &str,
        stake_e8s: u64,
        maturity_e8s: u64,
        created_timestamp_seconds: u64,
    ) -> SnsNeuronRow {
        SnsNeuronRow {
            neuron_id: neuron_id.to_string(),
            cached_neuron_stake_e8s: stake_e8s,
            maturity_e8s_equivalent: maturity_e8s,
            staked_maturity_e8s_equivalent: None,
            created_timestamp_seconds,
            created_at: created_timestamp_seconds.to_string(),
        }
    }
}
