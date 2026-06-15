use super::timestamp::current_timestamp_text;
use crate::{
    snapshot_cache::{SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt},
    sns::report::{
        SnsNeuronsRefreshRequest,
        source::{MainnetSns, SnsFetchRequest},
    },
};
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub(in crate::sns::report::neurons_cache) type SnsNeuronsRefreshAttempt =
    SnapshotRefreshAttempt<SnsNeuronsRefreshAttemptMetadata>;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(in crate::sns::report::neurons_cache) struct SnsNeuronsRefreshAttemptMetadata {
    pub(in crate::sns::report::neurons_cache::attempt) root_canister_id: String,
    pub(in crate::sns::report::neurons_cache::attempt) governance_canister_id: String,
}

pub(in crate::sns::report::neurons_cache) struct SnsNeuronsAttemptParts<'a> {
    pub(in crate::sns::report::neurons_cache) request: &'a SnsNeuronsRefreshRequest,
    pub(in crate::sns::report::neurons_cache) fetch_request: &'a SnsFetchRequest,
    pub(in crate::sns::report::neurons_cache) sns: &'a MainnetSns,
    pub(in crate::sns::report::neurons_cache) status: &'static str,
    pub(in crate::sns::report::neurons_cache) pages_fetched: u32,
    pub(in crate::sns::report::neurons_cache) rows_fetched: usize,
    pub(in crate::sns::report::neurons_cache) last_cursor: Option<String>,
    pub(in crate::sns::report::neurons_cache) last_error: Option<String>,
}

pub(in crate::sns::report::neurons_cache) fn attempt_from_parts(
    parts: SnsNeuronsAttemptParts<'_>,
) -> SnsNeuronsRefreshAttempt {
    SnsNeuronsRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: parts.request.network.clone(),
        source_endpoint: parts.request.source_endpoint.clone(),
        started_at: parts.fetch_request.fetched_at.clone(),
        updated_at: current_timestamp_text(&parts.fetch_request.fetched_at),
        metadata: SnsNeuronsRefreshAttemptMetadata {
            root_canister_id: parts.sns.root_canister_id.clone(),
            governance_canister_id: parts.sns.governance_canister_id.clone(),
        },
        status: parts.status.to_string(),
        page_size: parts.request.page_size,
        pages_fetched: parts.pages_fetched,
        rows_fetched: parts.rows_fetched,
        last_cursor: parts.last_cursor,
        last_error: parts.last_error,
    }
}
