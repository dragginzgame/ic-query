//! Boundary-level tests for query request and cached record ownership.

use crate::{
    diagnostic::StyleDiagnosticCode,
    query::{QueryExample, QueryRequest},
};

#[test]
fn records_query_through_query_owner() {
    let mut query = QueryExample::default();

    let report = query
        .record_query("sns-neurons", "https://icp-api.io")
        .expect("valid query request should succeed");

    assert_eq!(report.request().query_name(), "sns-neurons");
    assert_eq!(report.row().label(), "sns-neurons");
    assert_eq!(query.record_query_name("sns-neurons"), Some("sns-neurons"));
    assert_eq!(
        query.record_source_endpoint("sns-neurons"),
        Some("https://icp-api.io")
    );
}

#[test]
fn rejects_empty_query_name_without_matching_messages() {
    let err =
        QueryRequest::new("   ", "https://icp-api.io").expect_err("blank query names should fail");

    assert_eq!(err.code(), StyleDiagnosticCode::EmptyQueryName);
}
