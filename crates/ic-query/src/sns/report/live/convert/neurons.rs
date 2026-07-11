//! Module: sns::report::live::convert::neurons
//!
//! Responsibility: convert SNS governance neuron wire rows.
//! Does not own: governance transport, cache storage, or text rendering.
//! Boundary: maps live neuron rows and cursors into source/report models.

use crate::{
    sns::report::{SnsHostError, SnsNeuronRow, hex_bytes, live::types::SnsGovernanceNeuron},
    subnet_catalog::format_utc_timestamp_secs,
};

/// Convert one SNS governance neuron wire row into a report/cache row.
pub(in crate::sns::report::live) fn sns_neuron_row(
    neuron: SnsGovernanceNeuron,
) -> Result<SnsNeuronRow, SnsHostError> {
    let neuron_id = neuron.id.ok_or(SnsHostError::MissingNeuronId)?.id;
    if neuron_id.is_empty() {
        return Err(SnsHostError::InvalidNeuronId);
    }
    Ok(SnsNeuronRow {
        neuron_id: hex_bytes(&neuron_id),
        cached_neuron_stake_e8s: neuron.cached_neuron_stake_e8s,
        maturity_e8s_equivalent: neuron.maturity_e8s_equivalent,
        staked_maturity_e8s_equivalent: neuron.staked_maturity_e8s_equivalent,
        created_timestamp_seconds: neuron.created_timestamp_seconds,
        created_at: format_utc_timestamp_secs(neuron.created_timestamp_seconds),
    })
}
