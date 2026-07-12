//! Module: nns::node_provider
//!
//! Responsibility: assemble node-provider CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-provider arguments to the typed library API.

mod reports;
mod run;
mod spec;
#[cfg(test)]
pub(in crate::nns) mod test_helpers;

use ic_query::nns::node_provider::{NnsNodeProviderCacheRequest, NnsNodeProviderRefreshRequest};

pub(super) use run::run;

impl_leaf_refresh_cli_request!(NnsNodeProviderCacheRequest, NnsNodeProviderRefreshRequest);
