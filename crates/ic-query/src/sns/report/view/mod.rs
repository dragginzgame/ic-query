//! Module: sns::report::view
//!
//! Responsibility: apply report view options to in-memory SNS rows.
//! Does not own: command parsing, source fetching, cache loading, or rendering.
//! Boundary: transforms already-collected rows before report DTO assembly.

mod list;
mod neurons;
mod proposals;
#[cfg(test)]
mod tests;

pub(in crate::sns::report) use list::{filter_mainnet_sns_instances, sort_mainnet_sns_instances};
pub(in crate::sns::report) use neurons::sort_sns_neurons;
pub(in crate::sns::report) use proposals::{
    apply_sns_proposal_view, proposal_matches_before, proposal_matches_status,
    proposal_matches_topic,
};
