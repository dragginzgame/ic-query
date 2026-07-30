//! Module: nns::proposals::report::source::live
//!
//! Responsibility: query live NNS governance proposal APIs.
//! Does not own: report DTO assembly, cache publication, or text rendering.
//! Boundary: adapts source trait calls to candid queries against mainnet governance.

use crate::{
    nns::{
        LiveNnsSource, NnsSourceRequest,
        governance_query::query_nns_governance,
        proposals::report::{
            NnsProposalHostError, enforce_mainnet_network,
            model::{NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalStatusFilter},
            source::{NnsProposalSource, nns_proposal_row_from_info},
            wire::{
                NnsListProposalInfoRequest, NnsListProposalInfoResponse, NnsProposalId,
                NnsProposalInfo,
            },
        },
    },
    runtime::block_on_current_thread,
};

impl NnsProposalSource for LiveNnsSource {
    fn fetch_proposals(
        &self,
        request: &NnsSourceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> Result<Vec<NnsProposalRow>, NnsProposalHostError> {
        enforce_mainnet_network(&request.network)?;
        let include_status = status
            .governance_status_code()
            .into_iter()
            .collect::<Vec<_>>();
        let include_reward_status = reward_status
            .governance_reward_status_code()
            .into_iter()
            .collect::<Vec<_>>();
        let proposals = block_on_current_thread(fetch_nns_proposal_list_async(
            request,
            limit,
            before_proposal_id,
            &include_status,
            &include_reward_status,
        ))
        .map_err(NnsProposalHostError::Runtime)??;
        Ok(proposals
            .into_iter()
            .map(nns_proposal_row_from_info)
            .collect())
    }

    fn fetch_proposal(
        &self,
        request: &NnsSourceRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalRow, NnsProposalHostError> {
        enforce_mainnet_network(&request.network)?;
        Ok(nns_proposal_row_from_info(
            block_on_current_thread(fetch_nns_proposal_async(request, proposal_id))
                .map_err(NnsProposalHostError::Runtime)??,
        ))
    }
}

async fn fetch_nns_proposal_list_async(
    request: &NnsSourceRequest,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
    include_reward_status: &[i32],
) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
    let response: NnsListProposalInfoResponse = query_nns_governance(
        request,
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
    request: &NnsSourceRequest,
    proposal_id: u64,
) -> Result<NnsProposalInfo, NnsProposalHostError> {
    let proposal: Option<NnsProposalInfo> = query_nns_governance(
        request,
        "get_proposal_info",
        "ProposalId",
        "ProposalInfo",
        &proposal_id,
    )
    .await?;
    proposal.ok_or(NnsProposalHostError::ProposalNotFound { proposal_id })
}
