//! Module: sns::report::proposals_cache::model
//!
//! Responsibility: proposal snapshot cache model types.
//! Does not own: cache path construction, refresh orchestration, or rendering.
//! Boundary: defines complete proposal snapshot metadata, rows, and attempts.

use crate::snapshot_cache::{SnapshotEnvelope, SnapshotHeader, SnapshotRefreshAttempt};
use crate::sns::report::{
    SnsProposalRow,
    cache_attempt::SnsRefreshAttemptMetadata,
    cache_storage::{SnsCacheHeaderMetadata, SnsCacheMetadata},
};
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub(super) type SnsProposalsCache = SnapshotEnvelope<SnsCacheMetadata, SnsProposalsCacheRows>;

pub(super) type SnsProposalsCacheHeader = SnapshotHeader<SnsCacheHeaderMetadata>;

pub(super) type SnsProposalsRefreshAttempt = SnapshotRefreshAttempt<SnsRefreshAttemptMetadata>;

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
