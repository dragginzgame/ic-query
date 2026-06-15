use super::super::{attempt::SnsNeuronsAttemptContext, paths::SnsNeuronsCachePaths};
use crate::sns::report::{
    SnsNeuronsRefreshRequest,
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
