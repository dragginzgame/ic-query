//! Module: nns::proposals::report::cache::model
//!
//! Responsibility: complete NNS proposal snapshot cache DTOs.
//! Does not own: cache file IO, refresh orchestration, or text rendering.
//! Boundary: defines complete proposal snapshot metadata and rows.

use crate::{
    nns::{governance::NnsGovernanceCacheMetadata, proposals::report::model::NnsProposalRow},
    snapshot_cache::SnapshotEnvelope,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub(super) type NnsProposalCache =
    SnapshotEnvelope<NnsGovernanceCacheMetadata, NnsProposalCacheRows>;

///
/// NnsProposalCacheRows
///
/// Snapshot payload containing complete NNS proposal rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NnsProposalCacheRows {
    pub(super) collection_evidence:
        crate::nns::governance::collection::NnsGovernanceCollectionEvidence,
    pub(super) proposals: Vec<NnsProposalRow>,
}
