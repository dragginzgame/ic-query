//! Module: query::snapshot
//!
//! Responsibility: owner-local cached query record representation.
//! Does not own: JSON schema, refresh locks, or live source clients.
//! Boundary: records accepted query facts after request validation.

///
/// CachedQueryRecord
///
/// Owner-local cached query fact stored by the query module.
/// This type stays `pub(crate)` so callers must go through query reports and
/// owner queries instead of depending on storage internals.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CachedQueryRecord {
    query_name: String,
    source_endpoint: String,
}

impl CachedQueryRecord {
    /// Build one cached record from already-validated query input.
    #[must_use]
    pub(crate) fn new(query_name: &str, source_endpoint: &str) -> Self {
        Self {
            query_name: query_name.to_owned(),
            source_endpoint: source_endpoint.to_owned(),
        }
    }

    /// Return the query name covered by this cached record.
    #[must_use]
    pub(crate) fn query_name(&self) -> &str {
        &self.query_name
    }

    /// Return the source endpoint used by this cached record.
    #[must_use]
    pub(crate) fn source_endpoint(&self) -> &str {
        &self.source_endpoint
    }
}
