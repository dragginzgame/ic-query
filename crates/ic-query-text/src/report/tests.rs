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
