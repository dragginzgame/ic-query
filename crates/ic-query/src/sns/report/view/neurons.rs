//! Module: sns::report::view::neurons
//!
//! Responsibility: apply SNS neuron list view ordering.
//! Does not own: neuron fetching, cache loading, report assembly, or text rendering.
//! Boundary: sorts neuron rows without changing cache identity.

use crate::sns::report::{SnsNeuronRow, SnsNeuronsSort};

pub(in crate::sns::report) fn sort_sns_neurons(neurons: &mut [SnsNeuronRow], sort: SnsNeuronsSort) {
    match sort {
        SnsNeuronsSort::Api => {}
        SnsNeuronsSort::Id => neurons.sort_by(|left, right| left.neuron_id.cmp(&right.neuron_id)),
        SnsNeuronsSort::Stake => {
            sort_by_descending_value(neurons, |neuron| neuron.cached_neuron_stake_e8s);
        }
        SnsNeuronsSort::Maturity => {
            sort_by_descending_value(neurons, |neuron| neuron.maturity_e8s_equivalent);
        }
        SnsNeuronsSort::Created => {
            sort_by_descending_value(neurons, |neuron| neuron.created_timestamp_seconds);
        }
    }
}

fn sort_by_descending_value(neurons: &mut [SnsNeuronRow], value: impl Fn(&SnsNeuronRow) -> u64) {
    neurons.sort_by(|left, right| {
        value(right)
            .cmp(&value(left))
            .then_with(|| left.neuron_id.cmp(&right.neuron_id))
    });
}
