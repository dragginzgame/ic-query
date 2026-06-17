//! Module: sns::report::proposals_cache::collection::progress
//!
//! Responsibility: render proposal refresh progress text.
//! Does not own: paging state, attempt persistence, or terminal output.
//! Boundary: keeps proposal refresh progress wording local to collection refresh.

use crate::sns::report::source::MainnetSns;

pub(super) fn sns_proposals_progress_text(sns: &MainnetSns, pages: u32, rows: usize) -> String {
    format!(
        "refreshing SNS proposals for {}: pages={} rows={}",
        sns.name, pages, rows
    )
}
