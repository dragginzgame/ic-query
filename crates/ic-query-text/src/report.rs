//! Shared presentation of typed report provenance.

use ic_query::report::ReportProvenance;

/// Render source and complete-cache evidence in a stable human-facing order.
pub fn push_report_provenance_lines(lines: &mut Vec<String>, provenance: &ReportProvenance) {
    lines.push(format!("data_source: {}", provenance.data_source()));
    if let Some(cache_path) = provenance.cache_path() {
        lines.push(format!(
            "cache_path: {}",
            ic_host_text::escape::terminal_text(cache_path)
        ));
        lines.push("cache_complete: yes".to_string());
    }
}

#[cfg(test)]
mod tests {
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
}

///
/// TextOptions
///
/// Caller-selected detail and ballot visibility for human-facing report rendering.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextOptions {
    /// Expand human-facing report detail.
    pub verbose: bool,
    /// Include retained proposal ballots in human-facing output.
    pub show_ballots: bool,
}
