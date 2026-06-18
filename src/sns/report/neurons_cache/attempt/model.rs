//! Module: sns::report::neurons_cache::attempt::model
//!
//! Responsibility: define SNS neuron refresh-attempt sidecar models.
//! Does not own: sidecar IO, cache snapshots, refresh fetching, or rendering.
//! Boundary: maps refresh context and progress into the generic snapshot attempt envelope.

use crate::{
    snapshot_cache::{SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt},
    sns::report::{
        SnsNeuronsRefreshRequest,
        source::{MainnetSns, SnsFetchRequest},
    },
    subnet_catalog::format_utc_timestamp_secs,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub(in crate::sns::report::neurons_cache) type SnsNeuronsRefreshAttempt =
    SnapshotRefreshAttempt<SnsNeuronsRefreshAttemptMetadata>;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(in crate::sns::report::neurons_cache) struct SnsNeuronsRefreshAttemptMetadata {
    pub(in crate::sns::report::neurons_cache::attempt) root_canister_id: String,
    pub(in crate::sns::report::neurons_cache::attempt) governance_canister_id: String,
}

#[derive(Clone, Copy)]
pub(in crate::sns::report::neurons_cache) struct SnsNeuronsAttemptContext<'a> {
    pub(in crate::sns::report::neurons_cache) path: &'a Path,
    pub(in crate::sns::report::neurons_cache) request: &'a SnsNeuronsRefreshRequest,
    pub(in crate::sns::report::neurons_cache) fetch_request: &'a SnsFetchRequest,
    pub(in crate::sns::report::neurons_cache) sns: &'a MainnetSns,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report::neurons_cache) struct SnsNeuronsAttemptProgress {
    pub(in crate::sns::report::neurons_cache::attempt) pages_fetched: u32,
    pub(in crate::sns::report::neurons_cache::attempt) rows_fetched: usize,
    pub(in crate::sns::report::neurons_cache::attempt) last_cursor: Option<String>,
}

impl SnsNeuronsAttemptProgress {
    pub(in crate::sns::report::neurons_cache) const fn new(
        pages_fetched: u32,
        rows_fetched: usize,
        last_cursor: Option<String>,
    ) -> Self {
        Self {
            pages_fetched,
            rows_fetched,
            last_cursor,
        }
    }

    pub(in crate::sns::report::neurons_cache) const fn starting() -> Self {
        Self {
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
        }
    }
}

pub(in crate::sns::report::neurons_cache::attempt) struct SnsNeuronsAttemptParts<'a> {
    pub(in crate::sns::report::neurons_cache::attempt) context: SnsNeuronsAttemptContext<'a>,
    pub(in crate::sns::report::neurons_cache::attempt) status: &'static str,
    pub(in crate::sns::report::neurons_cache::attempt) progress: SnsNeuronsAttemptProgress,
    pub(in crate::sns::report::neurons_cache::attempt) last_error: Option<String>,
}

pub(in crate::sns::report::neurons_cache::attempt) fn attempt_from_parts(
    parts: SnsNeuronsAttemptParts<'_>,
) -> SnsNeuronsRefreshAttempt {
    SnsNeuronsRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: parts.context.request.network.clone(),
        source_endpoint: parts.context.request.source_endpoint.clone(),
        started_at: parts.context.fetch_request.fetched_at.clone(),
        updated_at: current_timestamp_text(&parts.context.fetch_request.fetched_at),
        metadata: SnsNeuronsRefreshAttemptMetadata {
            root_canister_id: parts.context.sns.root_canister_id.clone(),
            governance_canister_id: parts.context.sns.governance_canister_id.clone(),
        },
        status: parts.status.to_string(),
        page_size: parts.context.request.page_size,
        pages_fetched: parts.progress.pages_fetched,
        rows_fetched: parts.progress.rows_fetched,
        last_cursor: parts.progress.last_cursor,
        last_error: parts.last_error,
    }
}

fn current_timestamp_text(fallback: &str) -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| fallback.to_string(),
        |duration| format_utc_timestamp_secs(duration.as_secs()),
    )
}
