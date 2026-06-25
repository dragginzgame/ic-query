//! Module: sns::report::source::model::fetch
//!
//! Responsibility: shared SNS source fetch request model.
//! Does not own: endpoint validation, live transport, or report assembly.
//! Boundary: carries source endpoint and provenance for source calls.

///
/// SnsFetchRequest
///
/// Source-layer request shared by live SNS fetch helpers.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct SnsFetchRequest {
    pub(in crate::sns::report) endpoint: String,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) fetched_by: String,
}
