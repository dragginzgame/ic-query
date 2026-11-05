//! Module: progress
//!
//! Responsibility: render library progress events for the `icq` process.
//! Does not own: refresh orchestration, cache policy, or report construction.
//! Boundary: owns terminal detection and stderr output for query progress.

use ic_host_text::escape::terminal_text;
use ic_host_text::progress::ProgressLine as ProgressFrames;
use ic_query::QueryProgress;
use ic_query::QueryProgressCollection;
use ic_query::QueryProgressEvent;
use ic_query::QueryProgressState;
use std::{
    io::{self, IsTerminal, Write},
    path::Path,
};

///
/// StderrQueryProgress
///
/// CLI progress sink that renders structured query events on stderr.
///

#[derive(Debug)]
pub struct StderrQueryProgress {
    enabled: bool,
    frames: ProgressFrames,
    stderr: io::Stderr,
}

impl StderrQueryProgress {
    /// Create a CLI progress sink using the current stderr terminal state.
    pub fn new() -> Self {
        Self {
            enabled: io::stderr().is_terminal(),
            frames: ProgressFrames::new(),
            stderr: io::stderr(),
        }
    }

    fn update(&mut self, text: &str) {
        if self.enabled {
            let frame = self.frames.update(text);
            self.write_frame(&frame);
        }
    }

    fn finish(&mut self, text: &str) {
        if self.enabled {
            let frame = self.frames.finish(text);
            self.write_frame(&frame);
        }
    }

    fn clear(&mut self) {
        if self.enabled {
            let frame = self.frames.clear();
            if !frame.is_empty() {
                self.write_frame(&frame);
            }
        }
    }

    fn write_frame(&mut self, frame: &str) {
        let _ = self.stderr.write_all(frame.as_bytes());
        let _ = self.stderr.flush();
    }
}

impl QueryProgress for StderrQueryProgress {
    fn report(&mut self, event: QueryProgressEvent) {
        match event {
            QueryProgressEvent::CacheRefresh {
                component,
                path,
                source_endpoint,
            } => {
                self.clear();
                eprintln!(
                    "{}",
                    cache_refresh_text(&component, &path, &source_endpoint)
                );
            }
            QueryProgressEvent::PagedRefresh { progress, state } => {
                let text = paged_progress_text(&progress, state);
                if state == QueryProgressState::Running {
                    self.update(&text);
                } else {
                    self.finish(&text);
                }
            }
        }
    }
}

impl Drop for StderrQueryProgress {
    fn drop(&mut self) {
        self.clear();
    }
}

fn cache_refresh_text(component: &str, path: &Path, endpoint: &str) -> String {
    format!(
        "{} cache refresh required at {}; calling {} to refresh/create cache",
        terminal_text(component),
        terminal_text(&path.display().to_string()),
        terminal_text(endpoint),
    )
}

fn paged_progress_text(
    progress: &ic_query::PagedQueryProgress,
    state: QueryProgressState,
) -> String {
    let collection = match progress.collection {
        QueryProgressCollection::NnsNeurons => "NNS neurons",
        QueryProgressCollection::NnsProposals => "NNS proposals",
        QueryProgressCollection::SnsNeurons => "SNS neurons",
        QueryProgressCollection::IcrcAccountTransactions => "ICRC account transactions",
    };
    let target = progress
        .target
        .as_deref()
        .map_or_else(String::new, |target| {
            format!(" for {}", terminal_text(target))
        });
    let mut text = format!(
        "refreshing {collection}{target}: pages={} rows={}",
        progress.pages_fetched, progress.rows_fetched
    );
    if progress.collection != QueryProgressCollection::IcrcAccountTransactions {
        text.push_str(match state {
            QueryProgressState::Running => "",
            QueryProgressState::Complete => " complete",
            QueryProgressState::Stopped => " stopped before completion",
            QueryProgressState::Stalled => " stalled",
            QueryProgressState::Failed => " failed",
        });
    }
    text
}

#[cfg(test)]
mod tests;
