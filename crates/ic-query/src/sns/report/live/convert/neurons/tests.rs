use super::*;
use crate::sns::report::{
    SnsNeuronId, SnsPolicyObservationStatus,
    live::types::{SnsMetricsAccount, SnsMetricsSubaccount, SnsTopic},
};
use ic_platform::candid::contract::Principal;

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
    assert_eq!(row.created_timestamp_seconds, 1_700_000_000);
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

    assert_eq!(
        detail.neuron,
        sns_neuron_row(neuron(SnsGovernanceDissolveState::DissolveDelaySeconds(
            86_400
        )))
        .expect("list and detail retain the same native fields")
    );
    assert_eq!(detail.neuron.neuron_id, "01".repeat(32));
    assert_eq!(
        detail.permissions[0].principal.as_deref(),
        Some("2vxsx-fae")
    );
    assert_eq!(detail.permissions[0].permission_types[0], 9);
    assert_eq!(
        crate::sns::sns_neuron_permission_name(detail.permissions[0].permission_types[1]),
        "unknown"
    );
    let disbursement = &detail.disburse_maturity_in_progress[0];
    let account = disbursement
        .account_to_disburse_to
        .as_ref()
        .expect("destination account");
    assert_eq!(account.owner.as_deref(), Some("2vxsx-fae"));
    assert_eq!(account.subaccount_hex.as_deref(), Some(&*"ab".repeat(32)));
    assert_eq!(detail.followees[0].followee_neuron_ids[0], "02".repeat(32));
    let topic = &detail.topic_followees.as_ref().expect("topics")[0];
    assert_eq!(topic.topic.as_deref(), Some("governance"));
    assert_eq!(topic.followees[0].alias.as_deref(), Some("lead"));
    assert_eq!(
        detail.derived_policy_observations().0,
        SnsPolicyObservationStatus::Violated
    );
    assert_eq!(
        detail.derived_policy_observations().1,
        SnsPolicyObservationStatus::Violated
    );
}

#[test]
fn neuron_projections_reject_missing_or_empty_ids_before_other_evidence() {
    for id in [None, Some(SnsNeuronId { id: Vec::new() })] {
        let missing = id.is_none();
        let mut list = neuron(SnsGovernanceDissolveState::DissolveDelaySeconds(0));
        list.id = id.clone();
        let mut detail = detail_neuron();
        detail.id = id.clone();
        detail.permissions[0].principal = None;
        for result in [
            sns_neuron_row(list).map(|_| ()),
            mainnet_sns_neuron(detail).map(|_| ()),
        ] {
            let error = result.expect_err("reject invalid neuron identity first");
            if missing {
                assert!(matches!(error, SnsHostError::MissingNeuronId));
            } else {
                assert!(matches!(error, SnsHostError::InvalidNeuronId));
            }
        }
    }
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
