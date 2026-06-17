//! Module: sns::report::proposals_cache::model
//!
//! Responsibility: proposal snapshot cache model types.
//! Does not own: cache path construction, refresh orchestration, or rendering.
//! Boundary: defines complete proposal snapshot metadata, rows, and attempts.

use crate::snapshot_cache::{SnapshotEnvelope, SnapshotHeader, SnapshotRefreshAttempt};
use crate::sns::report::SnsProposalRow;
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub(super) type SnsProposalsCache =
    SnapshotEnvelope<SnsProposalsCacheMetadata, SnsProposalsCacheRows>;

pub(super) type SnsProposalsCacheHeader = SnapshotHeader<SnsProposalsCacheHeaderMetadata>;

pub(super) type SnsProposalsRefreshAttempt =
    SnapshotRefreshAttempt<SnsProposalsRefreshAttemptMetadata>;

///
/// SnsProposalsCacheMetadata
///
/// Snapshot metadata identifying the SNS covered by a complete proposal cache.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsProposalsCacheMetadata {
    pub(super) sns_wasm_canister_id: String,
    pub(super) id: usize,
    pub(super) name: String,
    pub(super) root_canister_id: String,
    pub(super) governance_canister_id: String,
}

///
/// SnsProposalsCacheRows
///
/// Snapshot payload containing complete SNS proposal rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsProposalsCacheRows {
    pub(super) proposals: Vec<SnsProposalRow>,
}

///
/// SnsProposalsCacheHeaderMetadata
///
/// Minimal metadata loaded while scanning proposal cache headers.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize)]
pub(super) struct SnsProposalsCacheHeaderMetadata {
    pub(super) id: usize,
}

///
/// SnsProposalsRefreshAttemptMetadata
///
/// Refresh-attempt metadata identifying the SNS proposal collection.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsProposalsRefreshAttemptMetadata {
    pub(super) root_canister_id: String,
    pub(super) governance_canister_id: String,
}

///
/// CompleteSnsProposals
///
/// Complete in-memory proposal collection produced by refresh paging.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CompleteSnsProposals {
    pub(super) proposals: Vec<SnsProposalRow>,
    pub(super) page_count: u32,
    pub(super) last_cursor: Option<String>,
}
