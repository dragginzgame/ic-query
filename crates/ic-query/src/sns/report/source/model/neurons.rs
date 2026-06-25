//! Module: sns::report::source::model::neurons
//!
//! Responsibility: source-layer SNS neuron models.
//! Does not own: governance transport, cache storage, sorting, or rendering.
//! Boundary: carries live neuron rows and pagination cursors into builders.

use crate::sns::report::SnsNeuronRow;
use candid::{CandidType, Deserialize};

///
/// MainnetSnsNeurons
///
/// Source-layer bounded SNS neuron listing.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsNeurons {
    pub(in crate::sns::report) neurons: Vec<SnsNeuronRow>,
}

///
/// MainnetSnsNeuronPage
///
/// Source-layer SNS neuron page used by complete snapshot refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsNeuronPage {
    pub(in crate::sns::report) neurons: Vec<SnsNeuronRow>,
    pub(in crate::sns::report) last_cursor: Option<SnsNeuronId>,
}

///
/// SnsNeuronId
///
/// Candid-compatible SNS neuron pagination cursor.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report) struct SnsNeuronId {
    pub(in crate::sns::report) id: Vec<u8>,
}
