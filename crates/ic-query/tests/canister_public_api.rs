#![cfg(all(feature = "canister", target_arch = "wasm32"))]
use ic_query::nns::governance::*;
use ic_query::nns::neuron::*;
use ic_query::nns::proposals::*;

#[test]
fn public_nns_governance_canister_source_builds_a_caller_runtime_future() {
    let request =
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_000);
    let future = build_nns_governance_economics_report_with_source(&request, &CanisterNnsSource);

    drop(future);
}

#[test]
fn public_nns_proposal_canister_source_builds_caller_runtime_futures() {
    let governance =
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_000);
    let list_request = NnsProposalListRequest::new(governance.clone(), 25);
    let detail_request = NnsProposalRequest::new(governance, 132_411);

    let collection_state =
        NnsProposalCollectionState::new(&list_request.governance, 25, 10).expect("valid state");
    let list_future = build_nns_proposal_list_report_with_source(&list_request, &CanisterNnsSource);
    let detail_future = build_nns_proposal_report_with_source(&detail_request, &CanisterNnsSource);
    let collection_future = advance_nns_proposal_collection_with_source(
        &list_request.governance,
        &collection_state,
        &CanisterNnsSource,
    );

    drop((list_future, detail_future, collection_future));
}

#[test]
fn public_nns_neuron_canister_source_builds_caller_runtime_futures() {
    let governance =
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_000);
    let list_request = NnsNeuronListRequest::new(governance.clone(), 25);
    let info_request = NnsNeuronInfoRequest::new(governance, 7);

    let collection_state =
        NnsNeuronCollectionState::new(&list_request.governance, 25, 10).expect("valid state");
    let list = build_nns_neuron_list_report_with_source(&list_request, &CanisterNnsSource);
    let info = build_nns_neuron_info_report_with_source(&info_request, &CanisterNnsSource);
    let collection = advance_nns_neuron_collection_with_source(
        &list_request.governance,
        &collection_state,
        &CanisterNnsSource,
    );

    drop((list, info, collection));
}
