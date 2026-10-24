//! Module: nns::proposals::report::source::canister
//!
//! Responsibility: call bounded NNS proposal APIs from replicated canister execution.
//! Does not own: scheduling, retries, persistence, report assembly, or view policy.
//! Boundary: maps shared canister transport responses into proposal source data.

use super::{
    NnsProposalSource, NnsProposalSourceFuture, nns_proposal_row_from_info, proposal_page_request,
};
use crate::nns::{
    governance::{
        CanisterNnsSource, NnsGovernanceRequest, NnsGovernanceSourceData, call_with_arg,
        canister_provenance,
    },
    proposals::report::{
        NnsProposalError,
        model::{NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalStatusFilter},
        wire::{NnsListProposalInfoResponse, NnsProposalInfo},
    },
};

impl NnsProposalSource for CanisterNnsSource {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            let provenance = canister_provenance(request)?;
            let response: NnsListProposalInfoResponse = call_with_arg(
                "list_proposals",
                "ListProposalInfoRequest",
                "ListProposalInfoResponse",
                &proposal_page_request(limit, before_proposal_id, status, reward_status),
            )
            .await?;
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
            let provenance = canister_provenance(request)?;
            let proposal: Option<NnsProposalInfo> = call_with_arg(
                "get_proposal_info",
                "ProposalId",
                "ProposalInfo",
                &proposal_id,
            )
            .await?;
            let proposal = proposal.ok_or(NnsProposalError::ProposalNotFound { proposal_id })?;
            Ok(NnsGovernanceSourceData::new(
                nns_proposal_row_from_info(proposal),
                provenance,
            ))
        })
    }
}
