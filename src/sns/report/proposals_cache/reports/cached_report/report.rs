//! Module: sns::report::proposals_cache::reports::cached_report::report
//!
//! Responsibility: project complete proposal caches into list reports.
//! Does not own: cache loading, refresh policy, or live source calls.
//! Boundary: constructs report DTOs after applying cache-backed view filters.

use super::super::super::model::SnsProposalsCache;
use super::filter::{proposal_matches_before, proposal_matches_status};
use crate::sns::report::{
    SnsProposalsReport, SnsProposalsRequest,
    assemble::{SnsProposalsReportParts, sns_proposals_report_from_parts},
    source::{MainnetSns, MainnetSnsList, MainnetSnsProposals},
};

pub(super) fn sns_proposals_report_from_cache(
    request: &SnsProposalsRequest,
    cache: SnsProposalsCache,
) -> SnsProposalsReport {
    let metadata = cache.metadata;
    let list = MainnetSnsList {
        network: request.network.clone(),
        sns_wasm_canister_id: metadata.sns_wasm_canister_id.clone(),
        fetched_at: cache.fetched_at,
        fetched_by: cache.fetched_by,
        source_endpoint: cache.source_endpoint,
        sns_instances: Vec::new(),
    };
    let sns = MainnetSns {
        id: metadata.id,
        name: metadata.name,
        description: None,
        url: None,
        root_canister_id: metadata.root_canister_id,
        governance_canister_id: metadata.governance_canister_id,
        ledger_canister_id: String::new(),
        swap_canister_id: String::new(),
        index_canister_id: String::new(),
        metadata_error: None,
    };
    let proposals = cache
        .data
        .proposals
        .into_iter()
        .filter(|proposal| proposal_matches_before(proposal, request.before_proposal_id))
        .filter(|proposal| proposal_matches_status(proposal, request.status))
        .take(usize::try_from(request.limit).unwrap_or(usize::MAX))
        .collect::<Vec<_>>();
    sns_proposals_report_from_parts(SnsProposalsReportParts {
        list,
        id: sns.id,
        sns,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        topic: request.topic,
        verbose: request.verbose,
        proposals: MainnetSnsProposals { proposals },
    })
}
