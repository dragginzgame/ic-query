//! Module: sns::report::view
//!
//! Responsibility: apply report view options to in-memory SNS rows.
//! Does not own: command parsing, source fetching, cache loading, or rendering.
//! Boundary: transforms already-collected rows before report DTO assembly.

mod proposals;

pub(in crate::sns::report) use proposals::sort_sns_proposal_rows;
