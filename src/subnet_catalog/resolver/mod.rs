//! Module: subnet_catalog::resolver
//!
//! Resolves subnet catalog inputs to concrete subnet subjects.

mod canister;
mod model;
mod prefix;
mod subnet;

pub(super) use canister::routing_range_sorts_after;
pub use model::{ResolveAs, ResolvedSubnet, ResolvedSubnetSubject};
