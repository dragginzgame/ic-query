use super::*;
use crate::{
    candid_decode::decode_reply,
    nns::neuron::report::{NnsNeuronError, source::governance_result},
};
use ic_platform::candid::contract::encode_one;

// Governance's external variant labels are part of the Candid contract.
#[derive(CandidType)]
#[candid_path("ic_platform::candid::contract")]
enum GovernanceReply<T> {
    Ok(T),
    Err(GovernanceError),
}

#[test]
fn index_reply_accepts_the_native_ok_variant() {
    let bytes = encode_one(GovernanceReply::Ok(NeuronIndexDataWire {
        neurons: Vec::new(),
    }))
    .unwrap();
    let reply = decode_reply::<GetNeuronIndexResultWire>(&bytes).unwrap();
    let page = governance_result(reply).unwrap();
    assert!(page.neurons.is_empty());
}

#[test]
fn boxed_info_reply_projects_the_native_error_variant() {
    let bytes = encode_one(GovernanceReply::<Box<NeuronInfoWire>>::Err(
        GovernanceError {
            error_type: 4,
            error_message: "fixture neuron is absent".to_string(),
        },
    ))
    .unwrap();
    let reply = decode_reply::<GetNeuronInfoResultWire>(&bytes).unwrap();
    assert!(matches!(
        governance_result(reply),
        Err(NnsNeuronError::GovernanceResponse { error_type: 4, message })
            if message == "fixture neuron is absent"
    ));
}
