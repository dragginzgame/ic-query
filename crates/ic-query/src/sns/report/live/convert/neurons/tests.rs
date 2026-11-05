use super::*;
use crate::sns::report::live::types::SnsGovernanceNeuronPermission;
use ic_platform::candid::contract::Principal;
fn wire() -> SnsGovernanceNeuron {
    SnsGovernanceNeuron {
        id: Some(SnsNeuronId { id: vec![1; 32] }),
        permissions: vec![SnsGovernanceNeuronPermission {
            principal: Some(Principal::management_canister()),
            permission_type: vec![5, 999],
        }],
        cached_neuron_stake_e8s: 125_000_000,
        neuron_fees_e8s: 10,
        maturity_e8s_equivalent: 2,
        staked_maturity_e8s_equivalent: Some(3),
        created_timestamp_seconds: 4,
        source_nns_neuron_id: None,
        auto_stake_maturity: Some(false),
        aging_since_timestamp_seconds: 5,
        dissolve_state: Some(SnsGovernanceDissolveState::DissolveDelaySeconds(60)),
        voting_power_percentage_multiplier: 100,
        vesting_period_seconds: None,
    }
}
#[test]
fn native_stake_projection_keeps_principals_raw_codes_and_distinct_amounts() {
    let neuron = sns_neuron_row(wire()).unwrap();
    assert_eq!(neuron.permissions[0].principal, "aaaaa-aa");
    assert_eq!(neuron.permissions[0].permission_types, [5, 999]);
    assert_eq!(neuron.cached_neuron_stake_e8s, 125_000_000);
    assert_eq!(neuron.neuron_fees_e8s, 10);
    assert_eq!(neuron.staked_maturity_e8s_equivalent, Some(3));
    assert_eq!(
        neuron.dissolve_state,
        Some(SnsNeuronDissolveState::DissolveDelaySeconds(60))
    );
}
#[test]
fn missing_permission_principals_are_not_fabricated() {
    let mut native = wire();
    native.permissions[0].principal = None;
    assert!(matches!(
        sns_neuron_row(native),
        Err(SnsHostError::MissingNeuronPermissionPrincipal {
            permission_index: 0,
            ..
        })
    ));
}
#[test]
fn native_queries_use_the_shared_stake_permission_contract() {
    use crate::sns::report::live::types::{
        GetNeuronResponse, GetNeuronResult, ListNeuronsResponse,
    };
    let bytes = ic_platform::candid::contract::encode_args((ListNeuronsResponse {
        neurons: vec![wire()],
    },))
    .unwrap();
    let response: ListNeuronsResponse = crate::candid_decode::decode_reply(&bytes).unwrap();
    assert_eq!(
        sns_neuron_row(response.neurons.into_iter().next().unwrap())
            .unwrap()
            .permissions[0]
            .permission_types,
        [5, 999]
    );
    let bytes = ic_platform::candid::contract::encode_args((GetNeuronResponse {
        result: Some(GetNeuronResult::Neuron(Box::new(wire()))),
    },))
    .unwrap();
    let response: GetNeuronResponse = crate::candid_decode::decode_reply(&bytes).unwrap();
    let Some(GetNeuronResult::Neuron(neuron)) = response.result else {
        panic!("expected neuron")
    };
    assert_eq!(
        sns_neuron_row(*neuron).unwrap().cached_neuron_stake_e8s,
        125_000_000
    );
}
