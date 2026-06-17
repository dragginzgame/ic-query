//! Module: query::request
//!
//! Responsibility: query request and report contracts.
//! Does not own: cache storage or text rendering.
//! Boundary: turns caller input into owner-approved query facts.

use crate::{diagnostic::StyleDiagnostic, query::CachedQueryRecord, report::ReportRow};

///
/// QueryRequest
///
/// Validated request to record one read-only query in the example owner module.
/// The request owns input normalization but does not persist cache state.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryRequest {
    query_name: String,
    source_endpoint: String,
}

impl QueryRequest {
    /// Build one validated query request.
    pub fn new(
        query_name: impl Into<String>,
        source_endpoint: impl Into<String>,
    ) -> Result<Self, StyleDiagnostic> {
        let query_name = query_name.into();
        let query_name = query_name.trim();
        let source_endpoint = source_endpoint.into();
        let source_endpoint = source_endpoint.trim();

        if query_name.is_empty() {
            return Err(StyleDiagnostic::empty_query_name());
        }

        if source_endpoint.is_empty() {
            return Err(StyleDiagnostic::empty_source_endpoint());
        }

        Ok(Self {
            query_name: query_name.to_owned(),
            source_endpoint: source_endpoint.to_owned(),
        })
    }

    /// Return the accepted query name.
    #[must_use]
    pub fn query_name(&self) -> &str {
        &self.query_name
    }

    /// Return the accepted source endpoint.
    #[must_use]
    pub fn source_endpoint(&self) -> &str {
        &self.source_endpoint
    }

    /// Convert this request into a cached record owned by the query module.
    #[must_use]
    pub(crate) fn cached_record(&self) -> CachedQueryRecord {
        CachedQueryRecord::new(&self.query_name, &self.source_endpoint)
    }
}

///
/// QueryReport
///
/// Result envelope returned after a query request has been accepted.
/// The report carries the validated request and selected report row without
/// exposing cache storage internals.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryReport {
    request: QueryRequest,
    row: ReportRow,
}

impl QueryReport {
    /// Build one report from an accepted request and report row.
    #[must_use]
    pub const fn new(request: QueryRequest, row: ReportRow) -> Self {
        Self { request, row }
    }

    /// Return the accepted request.
    #[must_use]
    pub const fn request(&self) -> &QueryRequest {
        &self.request
    }

    /// Return the report row chosen for the request.
    #[must_use]
    pub const fn row(&self) -> &ReportRow {
        &self.row
    }
}
