//! Module: nns::proposals::report::source::host
//!
//! Responsibility: query NNS proposals through the native replica adapter.
//! Does not own: report assembly, cache publication, or runtime blocking.
//! Boundary: maps native Governance responses into the portable source contract.

use super::{NnsProposalSource, NnsProposalSourceFuture, nns_proposal_row_from_info};
use crate::nns::{
    LiveNnsSource,
    governance::{NnsGovernanceError, NnsGovernanceRequest, NnsGovernanceSourceData, host_request},
    governance_query::query_nns_governance,
    proposals::report::{
        NnsProposalError,
        model::{NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalStatusFilter},
        wire::{
            NnsListProposalInfoRequest, NnsListProposalInfoResponse, NnsProposalId, NnsProposalInfo,
        },
    },
};

impl NnsProposalSource for LiveNnsSource {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            let (request, provenance) = host_request(request)?;
            let include_status = status
                .governance_status_code()
                .into_iter()
                .collect::<Vec<_>>();
            let include_reward_status = reward_status
                .governance_reward_status_code()
                .into_iter()
                .collect::<Vec<_>>();
            let response: NnsListProposalInfoResponse = query_nns_governance(
                &request,
                "list_proposals",
                "ListProposalInfoRequest",
                "ListProposalInfoResponse",
                &NnsListProposalInfoRequest {
                    include_reward_status,
                    omit_large_fields: Some(false),
                    before_proposal: before_proposal_id.map(|id| NnsProposalId { id }),
                    limit,
                    exclude_topic: Vec::new(),
                    include_all_manage_neuron_proposals: Some(true),
                    include_status,
                    return_self_describing_action: Some(false),
                },
            )
            .await
            .map_err(NnsGovernanceError::from)?;
            Ok(NnsGovernanceSourceData::new(
                response
                    .proposal_info
                    .into_iter()
                    .map(nns_proposal_row_from_info)
                    .collect(),
                provenance,
            ))
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        Box::pin(async move {
            let (request, provenance) = host_request(request)?;
            let proposal: Option<NnsProposalInfo> = query_nns_governance(
                &request,
                "get_proposal_info",
                "ProposalId",
                "ProposalInfo",
                &proposal_id,
            )
            .await
            .map_err(NnsGovernanceError::from)?;
            let proposal = proposal.ok_or(NnsProposalError::ProposalNotFound { proposal_id })?;
            Ok(NnsGovernanceSourceData::new(
                nns_proposal_row_from_info(proposal),
                provenance,
            ))
        })
    }
}
