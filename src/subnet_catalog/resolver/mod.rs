//! Module: subnet_catalog::resolver
//!
//! Responsibility: resolve subnet catalog inputs to concrete subnet subjects.
//!
//! Does not own: cache loading, command parsing, or report rendering.
//!
//! Boundary: exposes resolver result types and keeps matching helpers private to
//! subnet catalog report builders.

mod canister;
mod model;
mod prefix;
mod subnet;

pub(super) use canister::routing_range_sorts_after;
pub use model::{ResolveAs, ResolvedSubnet, ResolvedSubnetSubject};
