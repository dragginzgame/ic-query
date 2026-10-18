//! Module: sns::report::proposals_cache::model
//!
//! Responsibility: proposal snapshot cache model types.
//! Does not own: cache path construction, refresh orchestration, or rendering.
//! Boundary: defines complete proposal snapshot metadata, rows, and attempts.

use crate::{
    snapshot_cache::CompletePagedCollection,
    sns::report::{
        SnsProposalRow, cache_storage::SnsStoredCache,
        proposals_cache::paths::SnsProposalsCacheCollection,
    },
};
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub(super) type SnsProposalsCache = SnsStoredCache<SnsProposalsCacheCollection>;

///
/// SnsProposalsCacheRows
///
/// Snapshot payload containing complete SNS proposal rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::sns::report) struct SnsProposalsCacheRows {
    pub(super) proposals: Vec<SnsProposalRow>,
}

///
/// CompleteSnsProposals
///
/// Complete in-memory proposal collection produced by refresh paging.
///

pub(super) type CompleteSnsProposals = CompletePagedCollection<SnsProposalRow>;
