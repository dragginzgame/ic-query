//! Decode one shared neuron stake/permission projection for exact and paged queries.
use crate::sns::report::live::types::{NeuronId, SnsGovernanceDissolveState, SnsGovernanceNeuron};
use crate::sns::report::{
    SnsHostError, SnsNeuronDissolveState, SnsNeuronPermissionRow, SnsNeuronRow, hex_bytes,
};
pub(in crate::sns::report::live) fn sns_neuron_row(
    neuron: SnsGovernanceNeuron,
) -> Result<SnsNeuronRow, SnsHostError> {
    let neuron_id = neuron_id_text(neuron.id)?;
    Ok(SnsNeuronRow {
        neuron_id: neuron_id.clone(),
        permissions: neuron
            .permissions
            .into_iter()
            .enumerate()
            .map(|(index, p)| {
                Ok(SnsNeuronPermissionRow {
                    principal: p
                        .principal
                        .ok_or_else(|| SnsHostError::MissingNeuronPermissionPrincipal {
                            neuron_id: neuron_id.clone(),
                            permission_index: index,
                        })?
                        .to_text(),
                    permission_types: p.permission_type,
                })
            })
            .collect::<Result<Vec<_>, SnsHostError>>()?,
        cached_neuron_stake_e8s: neuron.cached_neuron_stake_e8s,
        maturity_e8s_equivalent: neuron.maturity_e8s_equivalent,
        staked_maturity_e8s_equivalent: neuron.staked_maturity_e8s_equivalent,
        created_timestamp_seconds: neuron.created_timestamp_seconds,
        source_nns_neuron_id: neuron.source_nns_neuron_id,
        auto_stake_maturity: neuron.auto_stake_maturity,
        aging_since_timestamp_seconds: neuron.aging_since_timestamp_seconds,
        dissolve_state: neuron.dissolve_state.map(sns_neuron_dissolve_state),
        voting_power_percentage_multiplier: neuron.voting_power_percentage_multiplier,
        vesting_period_seconds: neuron.vesting_period_seconds,
        neuron_fees_e8s: neuron.neuron_fees_e8s,
    })
}
fn neuron_id_text(id: Option<NeuronId>) -> Result<String, SnsHostError> {
    let id = id.ok_or(SnsHostError::MissingNeuronId)?.id;
    if id.is_empty() {
        return Err(SnsHostError::InvalidNeuronId);
    }
    Ok(hex_bytes(&id))
}
const fn sns_neuron_dissolve_state(state: SnsGovernanceDissolveState) -> SnsNeuronDissolveState {
    match state {
        SnsGovernanceDissolveState::DissolveDelaySeconds(seconds) => {
            SnsNeuronDissolveState::DissolveDelaySeconds(seconds)
        }
        SnsGovernanceDissolveState::WhenDissolvedTimestampSeconds(seconds) => {
            SnsNeuronDissolveState::WhenDissolvedTimestampSeconds(seconds)
        }
    }
}

#[cfg(test)]
mod tests;
