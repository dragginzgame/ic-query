//! Module: sns::report::model::reports::neurons
//!
//! Responsibility: group SNS neuron report DTOs.
//! Does not own: live neuron fetches, cache storage, sorting, or rendering.
//! Boundary: re-exports serializable neuron report models.

mod refresh;
mod report;
mod row;

pub use refresh::SnsNeuronsRefreshReport;
pub use report::SnsNeuronsReport;
pub use row::{SnsNeuronDissolveState, SnsNeuronRow};
