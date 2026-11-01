//! Module: nns::proposals::report::source::host
//!
//! Responsibility: query NNS proposals through the native replica adapter.
//! Does not own: report assembly, cache publication, or runtime blocking.
//! Boundary: maps native Governance responses into the portable source contract.

use super::{
    NnsProposalSource, NnsProposalSourceFuture, proposal_page_request, proposal_row_from_response,
    proposal_rows_from_response,
};
use crate::nns::{
    LiveNnsSource,
    governance::{NnsGovernanceRequest, NnsGovernanceSourceData, host_request},
    governance_query::query_nns_governance,
    proposals::report::{
        model::{NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalStatusFilter},
        wire::{NnsListProposalInfoResponse, NnsProposalInfo},
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
            let response: NnsListProposalInfoResponse = query_nns_governance(
                self,
                &request,
                "list_proposals",
                "ListProposalInfoRequest",
                "ListProposalInfoResponse",
                &proposal_page_request(limit, before_proposal_id, status, reward_status),
            )
            .await?;
            Ok(NnsGovernanceSourceData::new(
                proposal_rows_from_response(response),
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
                self,
                &request,
                "get_proposal_info",
                "ProposalId",
                "ProposalInfo",
                &proposal_id,
            )
            .await?;
            Ok(NnsGovernanceSourceData::new(
                proposal_row_from_response(proposal, proposal_id)?,
                provenance,
            ))
        })
    }
}
