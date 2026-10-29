//! Module: nns::neuron::report::cache::model
//!
//! Responsibility: NNS neuron snapshot cache and cache-report DTOs.
//! Does not own: cache file IO, refresh orchestration, or text rendering.
//! Boundary: defines complete public neuron snapshot rows and report shapes.

use crate::{
    nns::{
        governance::NnsGovernanceCacheMetadata,
        neuron::report::{collection::NnsNeuronCollectionState, model::NnsNeuronRow},
    },
    snapshot_cache::SnapshotEnvelope,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub(super) type NnsNeuronCache = SnapshotEnvelope<NnsGovernanceCacheMetadata, NnsNeuronCacheRows>;

///
/// NnsNeuronCacheRows
///
/// Complete public neuron rows stored in one snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NnsNeuronCacheRows {
    pub(super) collection_state: NnsNeuronCollectionState,
    pub(super) neurons: Vec<NnsNeuronRow>,
}
