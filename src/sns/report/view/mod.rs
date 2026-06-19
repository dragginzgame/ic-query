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

pub(in crate::sns::report) use list::sort_mainnet_sns_instances;
pub(in crate::sns::report) use neurons::sort_sns_neurons;
pub(in crate::sns::report) use proposals::{
    proposal_matches_before, proposal_matches_status, sort_sns_proposal_rows,
};
