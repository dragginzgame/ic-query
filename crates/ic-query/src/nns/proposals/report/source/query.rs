//! Typed Governance operations shared by the native and canister adapters.

use super::{
    NnsProposalSource, NnsProposalSourceFuture, proposal_page_request, proposal_row_from_response,
    proposal_rows_from_response,
};
use crate::nns::{
    governance::{GovernanceQuery, NnsGovernanceRequest, NnsGovernanceSourceData},
    proposals::report::{
        model::{NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalStatusFilter},
        wire::{NnsListProposalInfoResponse, NnsProposalInfo},
    },
};

impl<T: GovernanceQuery> NnsProposalSource for T {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            let response: NnsGovernanceSourceData<NnsListProposalInfoResponse> = self
                .fetch(
                    request,
                    "list_proposals",
                    "ListProposalInfoRequest",
                    "ListProposalInfoResponse",
                    (proposal_page_request(
                        limit,
                        before_proposal_id,
                        status,
                        reward_status,
                    ),),
                )
                .await?;
            Ok(NnsGovernanceSourceData::new(
                proposal_rows_from_response(response.value),
                response.provenance,
            ))
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        Box::pin(async move {
            let proposal: NnsGovernanceSourceData<Option<NnsProposalInfo>> = self
                .fetch(
                    request,
                    "get_proposal_info",
                    "ProposalId",
                    "ProposalInfo",
                    (proposal_id,),
                )
                .await?;
            Ok(NnsGovernanceSourceData::new(
                proposal_row_from_response(proposal.value, proposal_id)?,
                proposal.provenance,
            ))
        })
    }
}
