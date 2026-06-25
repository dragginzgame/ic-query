//! Module: sns::report::live::convert::neurons
//!
//! Responsibility: convert SNS governance neuron wire rows.
//! Does not own: governance transport, cache storage, or text rendering.
//! Boundary: maps live neuron rows and cursors into source/report models.

use crate::{
    sns::report::{SnsNeuronId, SnsNeuronRow, hex_bytes, live::types::SnsGovernanceNeuron},
    subnet_catalog::format_utc_timestamp_secs,
};

/// Convert one SNS governance neuron wire row into a report/cache row.
pub(in crate::sns::report::live) fn sns_neuron_row(neuron: SnsGovernanceNeuron) -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: neuron
            .id
            .map_or_else(|| "-".to_string(), |id| hex_bytes(&id.id)),
        cached_neuron_stake_e8s: neuron.cached_neuron_stake_e8s,
        maturity_e8s_equivalent: neuron.maturity_e8s_equivalent,
        staked_maturity_e8s_equivalent: neuron.staked_maturity_e8s_equivalent,
        created_timestamp_seconds: neuron.created_timestamp_seconds,
        created_at: format_utc_timestamp_secs(neuron.created_timestamp_seconds),
    }
}

/// Extract the pagination cursor from one SNS governance neuron wire row.
pub(in crate::sns::report::live) fn sns_neuron_cursor(
    neuron: &SnsGovernanceNeuron,
) -> Option<SnsNeuronId> {
    neuron.id.clone()
}
