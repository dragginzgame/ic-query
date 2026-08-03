//! Module: sns::report::live::convert::neurons
//!
//! Responsibility: convert SNS governance neuron wire rows.
//! Does not own: governance transport, cache storage, or text rendering.
//! Boundary: maps live neuron rows and cursors into source/report models.

use super::proposals::sns_topic_text;
use crate::{
    sns::report::{
        MainnetSnsNeuron, SnsHostError, SnsMaturityDisbursementRow, SnsNeuronAccount,
        SnsNeuronDetail, SnsNeuronDissolveState, SnsNeuronFolloweeRow, SnsNeuronFolloweesRow,
        SnsNeuronPermissionRow, SnsNeuronPermissionValue, SnsNeuronRow, SnsNeuronTopicFolloweesRow,
        SnsPolicyObservationStatus, SnsRewardCheckpointRow, hex_bytes,
        live::types::{
            SnsGovernanceDissolveState, SnsGovernanceFollowee, SnsGovernanceFollowees,
            SnsGovernanceFolloweesForTopic, SnsGovernanceMaturityDisbursement, SnsGovernanceNeuron,
            SnsGovernanceNeuronDetail, SnsGovernanceNeuronPermission, SnsGovernanceRewardNeuron,
            SnsGovernanceTopicFollowees,
        },
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

/// Convert one exact SNS Governance neuron into full detail source data.
pub(in crate::sns::report::live) fn mainnet_sns_neuron(
    neuron: SnsGovernanceNeuronDetail,
) -> Result<MainnetSnsNeuron, SnsHostError> {
    let SnsGovernanceNeuronDetail {
        id,
        staked_maturity_e8s_equivalent,
        permissions,
        maturity_e8s_equivalent,
        cached_neuron_stake_e8s,
        created_timestamp_seconds,
        source_nns_neuron_id,
        auto_stake_maturity,
        aging_since_timestamp_seconds,
        dissolve_state,
        voting_power_percentage_multiplier,
        vesting_period_seconds,
        disburse_maturity_in_progress,
        followees,
        topic_followees,
        neuron_fees_e8s,
    } = neuron;
    let neuron_id = id.as_ref().ok_or(SnsHostError::MissingNeuronId)?;
    let neuron_id_text = hex_bytes(&neuron_id.id);
    let neuron = sns_neuron_row(SnsGovernanceNeuron {
        id,
        staked_maturity_e8s_equivalent,
        maturity_e8s_equivalent,
        cached_neuron_stake_e8s,
        created_timestamp_seconds,
        source_nns_neuron_id,
        auto_stake_maturity,
        aging_since_timestamp_seconds,
        dissolve_state,
        voting_power_percentage_multiplier,
        vesting_period_seconds,
        neuron_fees_e8s,
    })?;
    let permissions = permissions
        .into_iter()
        .enumerate()
        .map(|(index, permission)| required_neuron_permission(&neuron_id_text, index, permission))
        .collect::<Result<Vec<_>, _>>()?;
    let mut detail = SnsNeuronDetail {
        neuron,
        permissions,
        disburse_maturity_in_progress: disburse_maturity_in_progress
            .into_iter()
            .map(maturity_disbursement)
            .collect(),
        followees: followees
            .into_iter()
            .map(|(function_id, followees)| legacy_followees(function_id, followees))
            .collect(),
        topic_followees: topic_followees.map(topic_followees_rows),
        maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
        manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
    };
    let (mint, staking) = detail.derived_policy_observations();
    detail.maturity_mint_conversion_observed_disabled = mint;
    detail.manual_maturity_staking_observed_disabled = staking;
    Ok(MainnetSnsNeuron { detail })
}

/// Convert one reward-checkpoint neuron projection into report evidence.
pub(in crate::sns::report::live) fn sns_reward_checkpoint_row(
    neuron: SnsGovernanceRewardNeuron,
) -> Result<SnsRewardCheckpointRow, SnsHostError> {
    let neuron_id = neuron.id.ok_or(SnsHostError::MissingNeuronId)?.id;
    if neuron_id.is_empty() {
        return Err(SnsHostError::InvalidNeuronId);
    }
    let maturity_e8s_equivalent = neuron.maturity_e8s_equivalent;
    let staked_maturity_e8s_equivalent = neuron.staked_maturity_e8s_equivalent;
    let combined_maturity_e8s_equivalent = maturity_e8s_equivalent
        .checked_add(staked_maturity_e8s_equivalent.unwrap_or(0))
        .ok_or(SnsHostError::RewardCheckpointArithmetic {
            field: "combined_maturity_e8s_equivalent",
        })?;
    let mut row = SnsRewardCheckpointRow {
        neuron_id: hex_bytes(&neuron_id),
        created_timestamp_seconds: neuron.created_timestamp_seconds,
        maturity_e8s_equivalent,
        staked_maturity_e8s_equivalent,
        combined_maturity_e8s_equivalent,
        auto_stake_maturity: neuron.auto_stake_maturity,
        permissions: neuron
            .permissions
            .into_iter()
            .map(neuron_permission)
            .collect(),
        disburse_maturity_in_progress: neuron
            .disburse_maturity_in_progress
            .into_iter()
            .map(maturity_disbursement)
            .collect(),
        maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
        manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
    };
    let (mint, staking) = row.derived_policy_observations();
    row.maturity_mint_conversion_observed_disabled = mint;
    row.manual_maturity_staking_observed_disabled = staking;
    Ok(row)
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
        permission_types: permission
            .permission_type
            .into_iter()
            .map(SnsNeuronPermissionValue::from_code)
            .collect(),
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
mod tests {
    use super::*;
    use crate::sns::report::{
        SnsNeuronId,
        live::types::{SnsMetricsAccount, SnsMetricsSubaccount, SnsTopic},
    };
    use candid::Principal;

    #[test]
    fn sns_neuron_conversion_preserves_fixed_size_native_fields() {
        let row = sns_neuron_row(neuron(SnsGovernanceDissolveState::DissolveDelaySeconds(
            86_400,
        )))
        .expect("convert neuron");

        assert_eq!(row.neuron_id, "01".repeat(32));
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

    #[test]
    fn exact_neuron_conversion_preserves_variable_native_evidence() {
        let detail = mainnet_sns_neuron(detail_neuron()).expect("convert detail");

        assert_eq!(detail.detail.neuron.neuron_id, "01".repeat(32));
        assert_eq!(
            detail.detail.permissions[0].principal.as_deref(),
            Some("2vxsx-fae")
        );
        assert_eq!(detail.detail.permissions[0].permission_types[0].code, 9);
        assert_eq!(
            detail.detail.permissions[0].permission_types[1].name,
            "unknown"
        );
        let disbursement = &detail.detail.disburse_maturity_in_progress[0];
        let account = disbursement
            .account_to_disburse_to
            .as_ref()
            .expect("destination account");
        assert_eq!(account.owner.as_deref(), Some("2vxsx-fae"));
        assert_eq!(account.subaccount_hex.as_deref(), Some(&*"ab".repeat(32)));
        assert_eq!(
            detail.detail.followees[0].followee_neuron_ids[0],
            "02".repeat(32)
        );
        let topic = &detail.detail.topic_followees.as_ref().expect("topics")[0];
        assert_eq!(topic.topic.as_deref(), Some("governance"));
        assert_eq!(topic.followees[0].alias.as_deref(), Some("lead"));
        assert_eq!(
            detail.detail.maturity_mint_conversion_observed_disabled,
            SnsPolicyObservationStatus::Violated
        );
        assert_eq!(
            detail.detail.manual_maturity_staking_observed_disabled,
            SnsPolicyObservationStatus::Violated
        );
    }

    #[test]
    fn exact_neuron_conversion_rejects_missing_permission_principal() {
        let mut neuron = detail_neuron();
        neuron.permissions[0].principal = None;

        assert!(matches!(
            mainnet_sns_neuron(neuron),
            Err(SnsHostError::MissingNeuronPermissionPrincipal {
                neuron_id,
                permission_index: 0,
            }) if neuron_id == "01".repeat(32)
        ));
    }

    #[test]
    fn reward_neuron_conversion_preserves_unassessable_and_disbursement_evidence() {
        let mut neuron = reward_neuron();
        neuron.permissions[0].principal = None;
        neuron.permissions[0].permission_type = vec![11];

        let row = sns_reward_checkpoint_row(neuron).expect("convert reward neuron");

        assert_eq!(row.neuron_id, "01".repeat(32));
        assert_eq!(row.maturity_e8s_equivalent, 10_000_000);
        assert_eq!(row.staked_maturity_e8s_equivalent, Some(5_000_000));
        assert_eq!(row.combined_maturity_e8s_equivalent, 15_000_000);
        assert_eq!(row.permissions[0].principal, None);
        assert_eq!(row.permissions[0].permission_types[0].code, 11);
        assert_eq!(row.permissions[0].permission_types[0].name, "unknown");
        let disbursement = &row.disburse_maturity_in_progress[0];
        let account = disbursement
            .account_to_disburse_to
            .as_ref()
            .expect("destination account");
        assert_eq!(account.owner.as_deref(), Some("2vxsx-fae"));
        assert_eq!(account.subaccount_hex.as_deref(), Some(&*"ab".repeat(32)));
        assert_eq!(
            row.maturity_mint_conversion_observed_disabled,
            SnsPolicyObservationStatus::Violated
        );
        assert_eq!(
            row.manual_maturity_staking_observed_disabled,
            SnsPolicyObservationStatus::Unassessable
        );
    }

    #[test]
    fn reward_neuron_conversion_rejects_combined_maturity_overflow() {
        let mut neuron = reward_neuron();
        neuron.maturity_e8s_equivalent = u64::MAX;
        neuron.staked_maturity_e8s_equivalent = Some(1);

        assert!(matches!(
            sns_reward_checkpoint_row(neuron),
            Err(SnsHostError::RewardCheckpointArithmetic {
                field: "combined_maturity_e8s_equivalent",
            })
        ));
    }

    fn neuron(dissolve_state: SnsGovernanceDissolveState) -> SnsGovernanceNeuron {
        SnsGovernanceNeuron {
            id: Some(SnsNeuronId { id: vec![1; 32] }),
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

    fn detail_neuron() -> SnsGovernanceNeuronDetail {
        SnsGovernanceNeuronDetail {
            id: Some(SnsNeuronId { id: vec![1; 32] }),
            staked_maturity_e8s_equivalent: Some(5_000_000),
            permissions: vec![SnsGovernanceNeuronPermission {
                principal: Some(Principal::anonymous()),
                permission_type: vec![9, 11],
            }],
            maturity_e8s_equivalent: 10_000_000,
            cached_neuron_stake_e8s: 100_000_000,
            created_timestamp_seconds: 1_700_000_000,
            source_nns_neuron_id: Some(42),
            auto_stake_maturity: Some(true),
            aging_since_timestamp_seconds: 1_700_000_100,
            dissolve_state: Some(SnsGovernanceDissolveState::DissolveDelaySeconds(86_400)),
            voting_power_percentage_multiplier: 100,
            vesting_period_seconds: Some(31_536_000),
            disburse_maturity_in_progress: vec![SnsGovernanceMaturityDisbursement {
                timestamp_of_disbursement_seconds: 1_700_000_200,
                amount_e8s: 1_000,
                account_to_disburse_to: Some(SnsMetricsAccount {
                    owner: Some(Principal::anonymous()),
                    subaccount: Some(SnsMetricsSubaccount {
                        subaccount: vec![0xab; 32],
                    }),
                }),
                finalize_disbursement_timestamp_seconds: Some(1_700_086_600),
            }],
            followees: vec![(
                1,
                SnsGovernanceFollowees {
                    followees: vec![SnsNeuronId { id: vec![2; 32] }],
                },
            )],
            topic_followees: Some(SnsGovernanceTopicFollowees {
                topic_id_to_followees: vec![(
                    5,
                    SnsGovernanceFolloweesForTopic {
                        followees: vec![SnsGovernanceFollowee {
                            neuron_id: Some(SnsNeuronId { id: vec![3; 32] }),
                            alias: Some("lead".to_string()),
                        }],
                        topic: Some(SnsTopic::Governance),
                    },
                )],
            }),
            neuron_fees_e8s: 10_000,
        }
    }

    fn reward_neuron() -> SnsGovernanceRewardNeuron {
        let detail = detail_neuron();
        SnsGovernanceRewardNeuron {
            id: detail.id,
            staked_maturity_e8s_equivalent: detail.staked_maturity_e8s_equivalent,
            permissions: detail.permissions,
            maturity_e8s_equivalent: detail.maturity_e8s_equivalent,
            created_timestamp_seconds: detail.created_timestamp_seconds,
            auto_stake_maturity: detail.auto_stake_maturity,
            disburse_maturity_in_progress: detail.disburse_maturity_in_progress,
        }
    }
}
