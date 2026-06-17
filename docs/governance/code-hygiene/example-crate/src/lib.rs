//! Module: lib
//!
//! Responsibility: documentation-only crate root for ic-query style examples.
//! Does not own: runtime behavior, crate API, or production query contracts.
//! Boundary: exposes a small query and report surface used only by docs.

pub mod diagnostic;
pub mod query;
pub mod report;

pub use diagnostic::{StyleDiagnostic, StyleDiagnosticCode};
pub use query::{QueryReport, QueryRequest};
pub use report::{ReportRow, ReportRowKind};
