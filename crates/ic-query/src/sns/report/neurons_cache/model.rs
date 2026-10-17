//! Module: sns::report::neurons_cache::model
//!
//! Responsibility: define SNS neuron cache rows and complete collection types.
//! Does not own: cache storage, refresh collection, report rendering, or CLI parsing.
//! Boundary: keeps persisted cache models separate from public report models.

use crate::{snapshot_cache::CompletePagedCollection, sns::report::SnsNeuronRow};
use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// SnsNeuronsCacheRows
///
/// Snapshot payload containing complete SNS neuron rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SnsNeuronsCacheRows {
    pub(super) neurons: Vec<SnsNeuronRow>,
}

///
/// CompleteSnsNeurons
///
/// Complete in-memory neuron collection produced by refresh paging.
///

pub(super) type CompleteSnsNeurons = CompletePagedCollection<SnsNeuronRow>;
