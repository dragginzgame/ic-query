//! Project admitted acquisition evidence into neuron reports.
use crate::report::CollectionSourceRequest;
use crate::{
    snapshot_cache::SnapshotEnvelope,
    sns::report::{SnsInstanceContext, SnsReportContext, cache_storage::SnsCacheMetadata},
};
impl SnsReportContext {
    pub(in crate::sns::report) fn from_snapshot<Data>(
        cache: SnapshotEnvelope<SnsCacheMetadata, Data>,
    ) -> (Self, Data) {
        (
            Self {
                instance: SnsInstanceContext {
                    network: cache.network,
                    fetched_at: cache.fetched_at,
                    source_endpoint: cache.source_endpoint,
                    fetched_by: cache.fetched_by,
                },
                governance_canister_id: cache.metadata.governance_canister_id,
            },
            cache.data,
        )
    }
    pub(in crate::sns::report) fn from_live(
        request: CollectionSourceRequest,
        governance_canister_id: String,
    ) -> Self {
        Self {
            instance: SnsInstanceContext {
                network: request.network,
                fetched_at: request.fetched_at,
                source_endpoint: request.endpoint,
                fetched_by: request.fetched_by,
            },
            governance_canister_id,
        }
    }
}
