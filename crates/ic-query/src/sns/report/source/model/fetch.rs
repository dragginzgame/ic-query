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
    /// Requested IC network identity.
    pub network: String,
    /// IC API endpoint used for source calls.
    pub endpoint: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// Collector identity recorded in report provenance.
    pub fetched_by: String,
}

impl SnsSourceRequest {
    /// Construct one SNS source request with explicit network and provenance.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}
