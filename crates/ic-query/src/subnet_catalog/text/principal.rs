//! Module: subnet_catalog::text::principal
//!
//! Responsibility: render compact principal text for subnet catalog human output.
//!
//! Does not own: principal parsing, subject resolution, or JSON output.
//!
//! Boundary: keeps abbreviated principal display separate from canonical principal
//! validation and storage.

/// Returns the compact prefix used for narrow principal columns.
pub(super) fn compact_principal(value: &str) -> String {
    crate::text_value::compact_text(value, 5)
}
