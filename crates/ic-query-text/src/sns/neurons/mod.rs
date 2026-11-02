//! Module: sns::neurons
//!
//! Responsibility: group SNS neuron text report renderers.
//! Does not own: neuron report construction, cache loading, or JSON output.
//! Boundary: renders neuron list, refresh, and cache status DTOs for humans.

mod list;

pub use crate::sns::neurons::list::sns_neurons_report_text;
