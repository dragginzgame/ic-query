//! Module: subnet_catalog::resolver
//!
//! Resolves subnet catalog inputs to concrete subnet subjects.

mod canister;
mod model;
mod prefix;
mod subnet;

pub use model::{ResolveAs, ResolvedCanisterRoute, ResolvedSubnet, ResolvedSubnetSubject};
