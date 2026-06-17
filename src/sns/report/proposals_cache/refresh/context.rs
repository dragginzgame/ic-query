//! Module: sns::report::proposals_cache::refresh::context
//!
//! Responsibility: hold resolved context for a locked SNS proposal refresh.
//! Does not own: source fetching, cache publishing, or attempt persistence.
//! Boundary: adapts refresh context fields into attempt-writer context.

use super::super::{attempt::SnsProposalsAttemptContext, paths::SnsProposalsCachePaths};
use crate::sns::report::{
    SnsProposalsRefreshRequest,
    source::{MainnetSns, MainnetSnsList, SnsFetchRequest},
};

///
/// SnsProposalsRefreshContext
///
/// Resolved context for one locked proposal snapshot refresh.
///

pub(super) struct SnsProposalsRefreshContext<'a> {
    pub(super) request: &'a SnsProposalsRefreshRequest,
    pub(super) fetch_request: &'a SnsFetchRequest,
    pub(super) list: MainnetSnsList,
    pub(super) id: usize,
    pub(super) sns: MainnetSns,
    pub(super) paths: SnsProposalsCachePaths,
    pub(super) replaced_existing_cache: bool,
}

impl SnsProposalsRefreshContext<'_> {
    pub(super) fn attempt_context(&self) -> SnsProposalsAttemptContext<'_> {
        SnsProposalsAttemptContext {
            path: &self.paths.attempt_path,
            request: self.request,
            fetch_request: self.fetch_request,
            sns: &self.sns,
        }
    }
}
