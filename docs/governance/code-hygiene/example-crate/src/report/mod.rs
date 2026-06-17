//! Module: report
//!
//! Responsibility: small report-row contract for style examples.
//! Does not own: CLI parsing, query validation, or cache persistence.
//! Boundary: names the row selected by an owner module.

mod text;

pub use text::{ReportRow, ReportRowKind};
