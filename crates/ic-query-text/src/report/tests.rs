use super::*;

#[test]
fn provenance_lines_preserve_source_order_and_escape_cache_paths() {
    let mut lines = Vec::new();
    push_report_provenance_lines(&mut lines, &ReportProvenance::Live);
    assert_eq!(lines, ["data_source: live"]);
    lines.clear();
    push_report_provenance_lines(
        &mut lines,
        &ReportProvenance::Cache {
            cache_path: "/cache/snapshot.json".to_string(),
        },
    );
    assert_eq!(
        lines,
        [
            "data_source: cache",
            "cache_path: /cache/snapshot.json",
            "cache_complete: yes"
        ]
    );
}

#[test]
fn failed_attempt_text_preserves_progress_and_contains_untrusted_evidence() {
    let attempt = ic_query::cache::SnapshotRefreshAttemptStatus {
        status: ic_query::cache::CacheRefreshAttemptStatus::Failed,
        started_at: "2026-11-05T00:00:00Z".to_string(),
        updated_at: "2026-11-05T00:00:01Z".to_string(),
        page_size: 100,
        pages_fetched: 2,
        rows_fetched: 150,
        last_cursor: Some("cursor\nforged: value".to_string()),
        last_error: Some("failure\nforged: value".to_string()),
    };
    let text = refresh_attempt_lines(&attempt).join("\n");
    assert!(text.contains("attempt_status: failed"));
    assert!(text.contains("attempt_rows_fetched: 150"));
    assert!(text.contains("attempt_last_cursor: cursor\\nforged: value"));
    assert!(text.contains("attempt_last_error: failure\\nforged: value"));
    assert!(!text.lines().any(|line| line.starts_with("forged:")));
    assert!(attempt.last_error.unwrap().contains('\n'));
}
