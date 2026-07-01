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
pub struct MainnetSnsNeurons {
    pub neurons: Vec<SnsNeuronRow>,
}

///
/// MainnetSnsNeuronPage
///
/// Source-layer SNS neuron page used by complete snapshot refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsNeuronPage {
    pub neurons: Vec<SnsNeuronRow>,
    pub last_cursor: Option<SnsNeuronId>,
}

///
/// SnsNeuronId
///
/// Candid-compatible SNS neuron pagination cursor.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SnsNeuronId {
    pub id: Vec<u8>,
}
