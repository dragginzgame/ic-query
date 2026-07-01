//! Module: sns::report::source::model::fetch
//!
//! Responsibility: shared SNS source fetch request model.
//! Does not own: endpoint validation, live transport, or report assembly.
//! Boundary: carries source endpoint and provenance for source calls.

///
/// SnsSourceRequest
///
/// Source request settings shared by SNS source-adapter calls.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsSourceRequest {
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl SnsSourceRequest {
    #[must_use]
    pub fn new(
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

pub(in crate::sns::report) type SnsFetchRequest = SnsSourceRequest;
