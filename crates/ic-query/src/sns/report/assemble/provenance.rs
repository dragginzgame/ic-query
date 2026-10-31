//! Module: sns::report::assemble::provenance
//!
//! Responsibility: derive validated SNS identity and acquisition context.
//! Does not own: cache loading, report DTOs, or text rendering.
//! Boundary: projects admitted live and cached identity into report context.

use crate::report::CollectionSourceRequest;
use crate::{
    snapshot_cache::SnapshotEnvelope,
    sns::report::{MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, cache_storage::SnsCacheMetadata},
};

///
/// SnsReportContext
///
/// Validated identity and acquisition metadata shared by SNS list and detail reports.
///

pub(in crate::sns::report) struct SnsReportContext {
    pub(in crate::sns::report) network: String,
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) source_endpoint: String,
    pub(in crate::sns::report) fetched_by: String,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) name: String,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
}

impl SnsReportContext {
    /// Consume admitted snapshot identity and return its separately owned rows.
    pub(in crate::sns::report) fn from_snapshot<Data>(
        cache: SnapshotEnvelope<SnsCacheMetadata, Data>,
    ) -> (Self, Data) {
        (
            Self {
                network: cache.network,
                sns_wasm_canister_id: cache.metadata.sns_wasm_canister_id,
                fetched_at: cache.fetched_at,
                source_endpoint: cache.source_endpoint,
                fetched_by: cache.fetched_by,
                id: cache.metadata.id,
                name: cache.metadata.name,
                root_canister_id: cache.metadata.root_canister_id,
                governance_canister_id: cache.metadata.governance_canister_id,
            },
            cache.data,
        )
    }

    /// Project the validated live source identity and collection provenance.
    pub(in crate::sns::report) fn from_live(
        fetch_request: CollectionSourceRequest,
        sns: MainnetSns,
    ) -> Self {
        Self {
            network: fetch_request.network,
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            fetched_at: fetch_request.fetched_at,
            source_endpoint: fetch_request.endpoint,
            fetched_by: fetch_request.fetched_by,
            id: sns.id,
            name: sns.name,
            root_canister_id: sns.canisters.root_canister_id,
            governance_canister_id: sns.canisters.governance_canister_id,
        }
    }
}
