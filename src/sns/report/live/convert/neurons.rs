use super::super::{
    super::{SnsNeuronRow, hex_bytes},
    SnsNeuronId,
    types::SnsGovernanceNeuron,
};
use crate::subnet_catalog::format_utc_timestamp_secs;

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

pub(in crate::sns::report::live) fn sns_neuron_cursor(
    neuron: &SnsGovernanceNeuron,
) -> Option<SnsNeuronId> {
    neuron.id.clone()
}
