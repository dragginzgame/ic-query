//! Module: nns::neuron
//!
//! Responsibility: expose public NNS neuron reporting contracts.
//! Does not own: NNS proposal reporting or Dashboard-derived neuron analytics.
//! Boundary: re-exports the direct Governance neuron report family.

mod report;

pub use report::*;
