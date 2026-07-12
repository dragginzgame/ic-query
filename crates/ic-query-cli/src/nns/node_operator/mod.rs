//! Module: nns::node_operator
//!
//! Responsibility: assemble node-operator CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-operator arguments to the typed library API.

mod reports;
mod run;
mod spec;
#[cfg(test)]
pub(in crate::nns) mod test_helpers;

use ic_query::nns::node_operator::{NnsNodeOperatorCacheRequest, NnsNodeOperatorRefreshRequest};

pub(super) use run::run;

impl_leaf_refresh_cli_request!(NnsNodeOperatorCacheRequest, NnsNodeOperatorRefreshRequest);
