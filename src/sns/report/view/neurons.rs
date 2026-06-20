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
