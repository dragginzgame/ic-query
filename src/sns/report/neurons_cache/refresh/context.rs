//! Module: sns::report::neurons_cache::refresh::context
//!
//! Responsibility: carry SNS neuron refresh state across lock, fetch, and publish steps.
//! Does not own: lookup, cache writes, attempt serialization, or report rendering.
//! Boundary: builds attempt contexts from one resolved refresh operation.

use crate::sns::report::{
    SnsNeuronsRefreshRequest,
    neurons_cache::{attempt::SnsNeuronsAttemptContext, paths::SnsNeuronsCachePaths},
    source::{MainnetSns, MainnetSnsList, SnsFetchRequest},
};

pub(super) struct SnsNeuronsRefreshContext<'a> {
    pub(super) request: &'a SnsNeuronsRefreshRequest,
    pub(super) fetch_request: &'a SnsFetchRequest,
    pub(super) list: MainnetSnsList,
    pub(super) id: usize,
    pub(super) sns: MainnetSns,
    pub(super) paths: SnsNeuronsCachePaths,
    pub(super) replaced_existing_cache: bool,
}

impl SnsNeuronsRefreshContext<'_> {
    pub(super) fn attempt_context(&self) -> SnsNeuronsAttemptContext<'_> {
        SnsNeuronsAttemptContext {
            path: &self.paths.attempt_path,
            request: self.request,
            fetch_request: self.fetch_request,
            sns: &self.sns,
        }
    }
}
