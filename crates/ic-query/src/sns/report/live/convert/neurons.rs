//! Module: sns::report::live::convert::neurons
//!
//! Responsibility: convert SNS governance neuron wire rows.
//! Does not own: governance transport, cache storage, or text rendering.
//! Boundary: maps live neuron rows and cursors into source/report models.

use crate::{
    sns::report::{
        SnsHostError, SnsNeuronDissolveState, SnsNeuronRow, hex_bytes,
        live::types::{SnsGovernanceDissolveState, SnsGovernanceNeuron},
    },
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
        source_nns_neuron_id: neuron.source_nns_neuron_id,
        auto_stake_maturity: neuron.auto_stake_maturity,
        aging_since_timestamp_seconds: neuron.aging_since_timestamp_seconds,
        dissolve_state: neuron.dissolve_state.map(sns_neuron_dissolve_state),
        voting_power_percentage_multiplier: neuron.voting_power_percentage_multiplier,
        vesting_period_seconds: neuron.vesting_period_seconds,
        neuron_fees_e8s: neuron.neuron_fees_e8s,
    })
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
mod tests {
    use super::*;
    use crate::sns::report::SnsNeuronId;

    #[test]
    fn sns_neuron_conversion_preserves_fixed_size_native_fields() {
        let row = sns_neuron_row(neuron(SnsGovernanceDissolveState::DissolveDelaySeconds(
            86_400,
        )))
        .expect("convert neuron");

        assert_eq!(row.neuron_id, "0102");
        assert_eq!(row.cached_neuron_stake_e8s, 100_000_000);
        assert_eq!(row.maturity_e8s_equivalent, 10_000_000);
        assert_eq!(row.staked_maturity_e8s_equivalent, Some(5_000_000));
        assert_eq!(row.created_timestamp_seconds, 1_700_000_000);
        assert_eq!(row.created_at, "2023-11-14T22:13:20Z");
        assert_eq!(row.source_nns_neuron_id, Some(42));
        assert_eq!(row.auto_stake_maturity, Some(true));
        assert_eq!(row.aging_since_timestamp_seconds, 1_700_000_100);
        assert_eq!(
            row.dissolve_state,
            Some(SnsNeuronDissolveState::DissolveDelaySeconds(86_400))
        );
        assert_eq!(row.voting_power_percentage_multiplier, 100);
        assert_eq!(row.vesting_period_seconds, Some(31_536_000));
        assert_eq!(row.neuron_fees_e8s, 10_000);
    }

    #[test]
    fn sns_neuron_conversion_preserves_dissolved_timestamp_alternative() {
        let row = sns_neuron_row(neuron(
            SnsGovernanceDissolveState::WhenDissolvedTimestampSeconds(1_800_000_000),
        ))
        .expect("convert neuron");

        assert_eq!(
            row.dissolve_state,
            Some(SnsNeuronDissolveState::WhenDissolvedTimestampSeconds(
                1_800_000_000
            ))
        );
    }

    fn neuron(dissolve_state: SnsGovernanceDissolveState) -> SnsGovernanceNeuron {
        SnsGovernanceNeuron {
            id: Some(SnsNeuronId { id: vec![1, 2] }),
            staked_maturity_e8s_equivalent: Some(5_000_000),
            maturity_e8s_equivalent: 10_000_000,
            cached_neuron_stake_e8s: 100_000_000,
            created_timestamp_seconds: 1_700_000_000,
            source_nns_neuron_id: Some(42),
            auto_stake_maturity: Some(true),
            aging_since_timestamp_seconds: 1_700_000_100,
            dissolve_state: Some(dissolve_state),
            voting_power_percentage_multiplier: 100,
            vesting_period_seconds: Some(31_536_000),
            neuron_fees_e8s: 10_000,
        }
    }
}
