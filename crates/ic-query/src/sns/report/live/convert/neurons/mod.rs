//! Module: sns::report::live::convert::neurons
//!
//! Responsibility: convert SNS governance neuron wire rows.
//! Does not own: governance transport, cache storage, or text rendering.
//! Boundary: maps live neuron rows and cursors into source/report models.

use super::proposals::sns_topic_text;
use crate::sns::report::{
    SnsHostError, SnsMaturityDisbursementRow, SnsNeuronAccount, SnsNeuronDetail,
    SnsNeuronDissolveState, SnsNeuronFolloweeRow, SnsNeuronFolloweesRow, SnsNeuronId,
    SnsNeuronPermissionRow, SnsNeuronRow, SnsNeuronTopicFolloweesRow, hex_bytes,
    live::types::{
        SnsGovernanceDissolveState, SnsGovernanceFollowee, SnsGovernanceFollowees,
        SnsGovernanceFolloweesForTopic, SnsGovernanceMaturityDisbursement, SnsGovernanceNeuron,
        SnsGovernanceNeuronDetail, SnsGovernanceNeuronPermission, SnsGovernanceTopicFollowees,
    },
};

macro_rules! base_neuron_row {
    ($neuron:ident) => {
        SnsNeuronRow {
            neuron_id: neuron_id_text($neuron.id)?,
            cached_neuron_stake_e8s: $neuron.cached_neuron_stake_e8s,
            maturity_e8s_equivalent: $neuron.maturity_e8s_equivalent,
            staked_maturity_e8s_equivalent: $neuron.staked_maturity_e8s_equivalent,
            created_timestamp_seconds: $neuron.created_timestamp_seconds,
            source_nns_neuron_id: $neuron.source_nns_neuron_id,
            auto_stake_maturity: $neuron.auto_stake_maturity,
            aging_since_timestamp_seconds: $neuron.aging_since_timestamp_seconds,
            dissolve_state: $neuron.dissolve_state.map(sns_neuron_dissolve_state),
            voting_power_percentage_multiplier: $neuron.voting_power_percentage_multiplier,
            vesting_period_seconds: $neuron.vesting_period_seconds,
            neuron_fees_e8s: $neuron.neuron_fees_e8s,
        }
    };
}

/// Convert one SNS governance neuron wire row into a report/cache row.
pub(in crate::sns::report::live) fn sns_neuron_row(
    neuron: SnsGovernanceNeuron,
) -> Result<SnsNeuronRow, SnsHostError> {
    Ok(base_neuron_row!(neuron))
}

/// Convert one exact SNS Governance neuron into full detail source data.
pub(in crate::sns::report::live) fn mainnet_sns_neuron(
    neuron: SnsGovernanceNeuronDetail,
) -> Result<SnsNeuronDetail, SnsHostError> {
    let row = base_neuron_row!(neuron);
    let permissions = neuron
        .permissions
        .into_iter()
        .enumerate()
        .map(|(index, permission)| required_neuron_permission(&row.neuron_id, index, permission))
        .collect::<Result<Vec<_>, _>>()?;
    let disburse_maturity_in_progress = neuron
        .disburse_maturity_in_progress
        .into_iter()
        .map(maturity_disbursement)
        .collect::<Vec<_>>();
    Ok(SnsNeuronDetail {
        neuron: row,
        permissions,
        disburse_maturity_in_progress,
        followees: neuron
            .followees
            .into_iter()
            .map(|(function_id, followees)| legacy_followees(function_id, followees))
            .collect(),
        topic_followees: neuron.topic_followees.map(topic_followees_rows),
    })
}

fn neuron_id_text(id: Option<SnsNeuronId>) -> Result<String, SnsHostError> {
    let id = id.ok_or(SnsHostError::MissingNeuronId)?.id;
    if id.is_empty() {
        return Err(SnsHostError::InvalidNeuronId);
    }
    Ok(hex_bytes(&id))
}

fn required_neuron_permission(
    neuron_id: &str,
    permission_index: usize,
    permission: SnsGovernanceNeuronPermission,
) -> Result<SnsNeuronPermissionRow, SnsHostError> {
    if permission.principal.is_none() {
        return Err(SnsHostError::MissingNeuronPermissionPrincipal {
            neuron_id: neuron_id.to_string(),
            permission_index,
        });
    }
    Ok(neuron_permission(permission))
}

fn neuron_permission(permission: SnsGovernanceNeuronPermission) -> SnsNeuronPermissionRow {
    SnsNeuronPermissionRow {
        principal: permission.principal.map(|principal| principal.to_text()),
        permission_types: permission.permission_type,
    }
}

fn maturity_disbursement(
    disbursement: SnsGovernanceMaturityDisbursement,
) -> SnsMaturityDisbursementRow {
    SnsMaturityDisbursementRow {
        timestamp_of_disbursement_seconds: disbursement.timestamp_of_disbursement_seconds,
        amount_e8s: disbursement.amount_e8s,
        account_to_disburse_to: disbursement.account_to_disburse_to.map(|account| {
            SnsNeuronAccount {
                owner: account.owner.map(|owner| owner.to_text()),
                subaccount_hex: account
                    .subaccount
                    .map(|subaccount| hex_bytes(&subaccount.subaccount)),
            }
        }),
        finalize_disbursement_timestamp_seconds: disbursement
            .finalize_disbursement_timestamp_seconds,
    }
}

fn legacy_followees(function_id: u64, followees: SnsGovernanceFollowees) -> SnsNeuronFolloweesRow {
    SnsNeuronFolloweesRow {
        function_id,
        followee_neuron_ids: followees
            .followees
            .into_iter()
            .map(|followee| hex_bytes(&followee.id))
            .collect(),
    }
}

fn topic_followees_rows(
    topic_followees: SnsGovernanceTopicFollowees,
) -> Vec<SnsNeuronTopicFolloweesRow> {
    topic_followees
        .topic_id_to_followees
        .into_iter()
        .map(|(topic_code, followees)| topic_followees_row(topic_code, followees))
        .collect()
}

fn topic_followees_row(
    topic_code: i32,
    followees: SnsGovernanceFolloweesForTopic,
) -> SnsNeuronTopicFolloweesRow {
    SnsNeuronTopicFolloweesRow {
        topic_code,
        topic: followees
            .topic
            .map(|topic| sns_topic_text(topic).to_string()),
        followees: followees
            .followees
            .into_iter()
            .map(topic_followee)
            .collect(),
    }
}

fn topic_followee(followee: SnsGovernanceFollowee) -> SnsNeuronFolloweeRow {
    SnsNeuronFolloweeRow {
        neuron_id: followee.neuron_id.map(|neuron_id| hex_bytes(&neuron_id.id)),
        alias: followee.alias,
    }
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
