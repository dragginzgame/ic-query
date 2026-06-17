//! Module: sns::report::proposals_cache::reports::cache_projection
//!
//! Responsibility: project complete proposal caches into report-building parts.
//! Does not own: cache loading, view filtering, or text rendering.
//! Boundary: keeps cache metadata conversion shared by list and detail reports.

use super::super::model::SnsProposalsCache;
use crate::sns::report::{
    SnsProposalRow,
    source::{MainnetSns, MainnetSnsList},
};

pub(super) struct SnsProposalsCacheProjection {
    pub(super) list: MainnetSnsList,
    pub(super) id: usize,
    pub(super) sns: MainnetSns,
    pub(super) proposals: Vec<SnsProposalRow>,
}

pub(super) fn project_sns_proposals_cache(cache: SnsProposalsCache) -> SnsProposalsCacheProjection {
    let metadata = cache.metadata;
    let list = MainnetSnsList {
        network: cache.network,
        sns_wasm_canister_id: metadata.sns_wasm_canister_id.clone(),
        fetched_at: cache.fetched_at,
        fetched_by: cache.fetched_by,
        source_endpoint: cache.source_endpoint,
        sns_instances: Vec::new(),
    };
    let id = metadata.id;
    let sns = MainnetSns {
        id,
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
    SnsProposalsCacheProjection {
        list,
        id,
        sns,
        proposals: cache.data.proposals,
    }
}
