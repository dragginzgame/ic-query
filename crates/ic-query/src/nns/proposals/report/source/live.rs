//! Module: nns::proposals::report::source::live
//!
//! Responsibility: query live NNS governance proposal APIs.
//! Does not own: report DTO assembly, cache publication, or text rendering.
//! Boundary: adapts source trait calls to candid queries against mainnet governance.

use crate::{
    nns::proposals::report::{
        MAINNET_GOVERNANCE_CANISTER_ID, NnsProposalHostError,
        source::{NnsProposalFetchRequest, NnsProposalSource},
        wire::{
            NnsListProposalInfoRequest, NnsListProposalInfoResponse, NnsProposalId, NnsProposalInfo,
        },
    },
    runtime::block_on_current_thread,
};
use candid::{CandidType, Deserialize, Principal};
use ic_agent::Agent;

///
/// LiveNnsProposalSource
///
/// Live source backed by the mainnet NNS governance canister.
///

pub(in crate::nns::proposals::report) struct LiveNnsProposalSource;

impl NnsProposalSource for LiveNnsProposalSource {
    fn fetch_proposals(
        &self,
        request: &NnsProposalFetchRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        include_reward_status: &[i32],
    ) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
        block_on_current_thread(fetch_nns_proposal_list_async(
            request,
            limit,
            before_proposal_id,
            include_status,
            include_reward_status,
        ))
        .map_err(NnsProposalHostError::Runtime)?
    }

    fn fetch_proposal(
        &self,
        request: &NnsProposalFetchRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalInfo, NnsProposalHostError> {
        block_on_current_thread(fetch_nns_proposal_async(request, proposal_id))
            .map_err(NnsProposalHostError::Runtime)?
    }
}

async fn fetch_nns_proposal_list_async(
    request: &NnsProposalFetchRequest,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
    include_reward_status: &[i32],
) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
    let agent = nns_agent(&request.endpoint)?;
    let governance_canister = governance_canister()?;
    let response: NnsListProposalInfoResponse = query_canister(
        &agent,
        &governance_canister,
        "list_proposals",
        "ListProposalInfoRequest",
        "ListProposalInfoResponse",
        &NnsListProposalInfoRequest {
            include_reward_status: include_reward_status.to_vec(),
            omit_large_fields: Some(false),
            before_proposal: before_proposal_id.map(|id| NnsProposalId { id }),
            limit,
            exclude_topic: Vec::new(),
            include_all_manage_neuron_proposals: Some(true),
            include_status: include_status.to_vec(),
            return_self_describing_action: Some(false),
        },
    )
    .await?;
    Ok(response.proposal_info)
}

async fn fetch_nns_proposal_async(
    request: &NnsProposalFetchRequest,
    proposal_id: u64,
) -> Result<NnsProposalInfo, NnsProposalHostError> {
    let agent = nns_agent(&request.endpoint)?;
    let governance_canister = governance_canister()?;
    let proposal: Option<NnsProposalInfo> = query_canister(
        &agent,
        &governance_canister,
        "get_proposal_info",
        "ProposalId",
        "ProposalInfo",
        &proposal_id,
    )
    .await?;
    proposal.ok_or(NnsProposalHostError::ProposalNotFound { proposal_id })
}

async fn query_canister<Arg, Response>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, NnsProposalHostError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|err| NnsProposalHostError::CandidEncode {
        message: request_message,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| NnsProposalHostError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| NnsProposalHostError::CandidDecode {
        message: response_message,
        reason: err.to_string(),
    })
}

fn nns_agent(endpoint: &str) -> Result<Agent, NnsProposalHostError> {
    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|err| NnsProposalHostError::AgentBuild {
            endpoint: endpoint.to_string(),
            reason: err.to_string(),
        })
}

fn governance_canister() -> Result<Principal, NnsProposalHostError> {
    Principal::from_text(MAINNET_GOVERNANCE_CANISTER_ID).map_err(|err| {
        NnsProposalHostError::CandidDecode {
            message: "governance_canister_id",
            reason: err.to_string(),
        }
    })
}
