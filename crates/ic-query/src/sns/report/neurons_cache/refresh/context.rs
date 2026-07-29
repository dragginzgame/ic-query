//! Module: sns::report::neurons_cache::refresh::context
//!
//! Responsibility: carry SNS neuron refresh state across lock, fetch, and publish steps.
//! Does not own: lookup, cache writes, attempt serialization, or report rendering.
//! Boundary: builds attempt contexts from one resolved refresh operation.

use crate::sns::report::{
    SnsNeuronsRefreshRequest,
    cache_attempt::SnsRefreshAttemptContext,
    neurons_cache::paths::SnsNeuronsCachePaths,
    source::{MainnetSns, MainnetSnsList, SnsSourceRequest},
};

///
/// SnsNeuronsRefreshContext
///
/// Resolved context for one locked neuron snapshot refresh.
///

pub(super) struct SnsNeuronsRefreshContext<'a> {
    pub(super) request: &'a SnsNeuronsRefreshRequest,
    pub(super) fetch_request: &'a SnsSourceRequest,
    pub(super) list: MainnetSnsList,
    pub(super) id: usize,
    pub(super) sns: MainnetSns,
    pub(super) paths: SnsNeuronsCachePaths,
    pub(super) replaced_existing_cache: bool,
}

impl SnsNeuronsRefreshContext<'_> {
    pub(super) fn attempt_context(&self) -> SnsRefreshAttemptContext<'_> {
        SnsRefreshAttemptContext {
            path: &self.paths.attempt_path,
            request: self.request,
            fetch_request: self.fetch_request,
            sns: &self.sns,
        }
    }
}
