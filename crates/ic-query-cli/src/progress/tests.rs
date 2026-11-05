use super::*;

#[test]
fn paging_text_has_one_lifecycle_owner_and_preserves_account_history_wording() {
    let mut progress = ic_query::PagedQueryProgress {
        collection: QueryProgressCollection::SnsNeurons,
        target: Some("Example SNS".to_string()),
        pages_fetched: 2,
        rows_fetched: 3,
    };
    for (state, suffix) in [
        (QueryProgressState::Running, ""),
        (QueryProgressState::Complete, " complete"),
        (QueryProgressState::Failed, " failed"),
        (QueryProgressState::Stopped, " stopped before completion"),
        (QueryProgressState::Stalled, " stalled"),
    ] {
        assert_eq!(
            paged_progress_text(&progress, state),
            format!("refreshing SNS neurons for Example SNS: pages=2 rows=3{suffix}")
        );
        let account = ic_query::PagedQueryProgress {
            collection: QueryProgressCollection::IcrcAccountTransactions,
            target: None,
            ..progress.clone()
        };
        assert_eq!(
            paged_progress_text(&account, state),
            "refreshing ICRC account transactions: pages=2 rows=3"
        );
    }
    progress.target = Some("unsafe\x1b[31m".to_string());
    assert!(!paged_progress_text(&progress, QueryProgressState::Running).contains('\x1b'));
}

#[test]
fn cache_refresh_progress_contains_untrusted_fields_on_one_line() {
    let path = Path::new("/cache\nforged\u{1b}");
    let text = cache_refresh_text(
        "component\nforged",
        path,
        "https://example.invalid/\u{1b}[31m",
    );
    assert!(!text.chars().any(char::is_control));
    assert!(text.contains("component\\nforged cache refresh required"));
    assert!(text.contains("/cache\\nforged"));
    assert!(text.contains("calling https://example.invalid/"));
}
