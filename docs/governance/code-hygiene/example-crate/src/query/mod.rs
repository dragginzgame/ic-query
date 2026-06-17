//! Module: query
//!
//! Responsibility: query request example and owner-local cache state.
//! Does not own: CLI parsing, live host calls, or text rendering.
//! Boundary: validates query requests before recording cached query facts.

mod request;
mod snapshot;

#[cfg(test)]
mod tests;

use crate::{
    diagnostic::StyleDiagnostic,
    report::{ReportRow, ReportRowKind},
};
use std::collections::BTreeMap;

pub use request::{QueryReport, QueryRequest};
pub(crate) use snapshot::CachedQueryRecord;

const MAX_SOURCE_ENDPOINT_BYTES: usize = 256;

///
/// QueryExample
///
/// Owner-local query example used to demonstrate cache ownership.
/// The query module owns normalized query facts; report code receives report
/// rows instead of reconstructing state from cache internals.
///

#[derive(Default)]
pub struct QueryExample {
    records: BTreeMap<String, CachedQueryRecord>,
}

impl QueryExample {
    /// Record one query request and return the report row selected for it.
    pub fn record_query(
        &mut self,
        query_name: impl Into<String>,
        source_endpoint: impl Into<String>,
    ) -> Result<QueryReport, StyleDiagnostic> {
        let request = QueryRequest::new(query_name, source_endpoint)?;
        let record = request.cached_record();
        let row = ReportRow::new(ReportRowKind::CacheRefresh, request.query_name())?;

        self.records.insert(request.query_name().to_owned(), record);

        Ok(QueryReport::new(request, row))
    }

    /// Return the cached source endpoint for one query when it is known.
    #[must_use]
    pub fn record_source_endpoint(&self, query_name: &str) -> Option<&str> {
        self.records
            .get(query_name)
            .map(CachedQueryRecord::source_endpoint)
    }

    /// Return the query name stored for one cache key.
    #[must_use]
    pub fn record_query_name(&self, query_name: &str) -> Option<&str> {
        self.records
            .get(query_name)
            .map(CachedQueryRecord::query_name)
    }

    /// Return a read-only report row without mutating cached state.
    pub fn read_row(&self, query_name: &str) -> Result<ReportRow, StyleDiagnostic> {
        ReportRow::new(ReportRowKind::CacheRead, query_name)
    }

    /// Return the example source-endpoint bound used by query callers.
    #[must_use]
    pub const fn max_source_endpoint_bytes() -> usize {
        MAX_SOURCE_ENDPOINT_BYTES
    }
}
