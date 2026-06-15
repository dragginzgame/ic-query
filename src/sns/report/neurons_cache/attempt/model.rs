use super::timestamp::current_timestamp_text;
use crate::sns::report::{
    SnsNeuronsRefreshRequest,
    source::{MainnetSns, SnsFetchRequest},
};
use serde::{Deserialize as SerdeDeserialize, Serialize};

const SNS_NEURONS_REFRESH_ATTEMPT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(in crate::sns::report::neurons_cache) struct SnsNeuronsRefreshAttempt {
    pub(in crate::sns::report::neurons_cache::attempt) schema_version: u32,
    pub(in crate::sns::report::neurons_cache::attempt) network: String,
    pub(in crate::sns::report::neurons_cache::attempt) source_endpoint: String,
    pub(in crate::sns::report::neurons_cache::attempt) started_at: String,
    pub(in crate::sns::report::neurons_cache::attempt) updated_at: String,
    pub(in crate::sns::report::neurons_cache::attempt) root_canister_id: String,
    pub(in crate::sns::report::neurons_cache::attempt) governance_canister_id: String,
    pub(in crate::sns::report::neurons_cache::attempt) status: String,
    pub(in crate::sns::report::neurons_cache::attempt) page_size: u32,
    pub(in crate::sns::report::neurons_cache::attempt) pages_fetched: u32,
    pub(in crate::sns::report::neurons_cache::attempt) rows_fetched: usize,
    pub(in crate::sns::report::neurons_cache::attempt) last_cursor: Option<String>,
    pub(in crate::sns::report::neurons_cache::attempt) last_error: Option<String>,
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
        schema_version: SNS_NEURONS_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: parts.request.network.clone(),
        source_endpoint: parts.request.source_endpoint.clone(),
        started_at: parts.fetch_request.fetched_at.clone(),
        updated_at: current_timestamp_text(&parts.fetch_request.fetched_at),
        root_canister_id: parts.sns.root_canister_id.clone(),
        governance_canister_id: parts.sns.governance_canister_id.clone(),
        status: parts.status.to_string(),
        page_size: parts.request.page_size,
        pages_fetched: parts.pages_fetched,
        rows_fetched: parts.rows_fetched,
        last_cursor: parts.last_cursor,
        last_error: parts.last_error,
    }
}
