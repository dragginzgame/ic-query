//! Module: progress
//!
//! Responsibility: render library progress events for the `icq` process.
//! Does not own: refresh orchestration, cache policy, or report construction.
//! Boundary: owns terminal detection and stderr output for query progress.

use ic_query::{
    QueryProgress, QueryProgressEvent, QueryProgressState, subnet_catalog::MAINNET_NETWORK,
};
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
    line: ProgressLine,
}

impl StderrQueryProgress {
    /// Create a CLI progress sink using the current stderr terminal state.
    pub fn new() -> Self {
        Self {
            line: ProgressLine::stderr(),
        }
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
                self.line.clear();
                eprintln!(
                    "{component} cache missing at {}; calling {source_endpoint} to refresh/create cache",
                    path.display()
                );
            }
            QueryProgressEvent::PagedRefresh { text, state } => {
                if state == QueryProgressState::Running {
                    self.line.update(&text);
                } else {
                    self.line.finish(&text);
                }
            }
        }
    }
}

/// Announce a missing mainnet cache before the CLI starts its live refresh.
pub fn announce_missing_mainnet_cache(
    network: &str,
    component: &str,
    path: &Path,
    source_endpoint: &str,
) {
    if network != MAINNET_NETWORK || path.is_file() {
        return;
    }
    let mut progress = StderrQueryProgress::new();
    progress.report(QueryProgressEvent::CacheRefresh {
        component: component.to_string(),
        path: path.to_path_buf(),
        source_endpoint: source_endpoint.to_string(),
    });
}

///
/// ProgressLine
///
/// Same-line stderr writer used by the CLI progress sink.
///

#[derive(Debug)]
struct ProgressLine {
    enabled: bool,
    last_width: usize,
    stderr: io::Stderr,
}

impl ProgressLine {
    fn stderr() -> Self {
        Self {
            enabled: io::stderr().is_terminal(),
            last_width: 0,
            stderr: io::stderr(),
        }
    }

    fn update(&mut self, text: &str) {
        if !self.enabled {
            return;
        }
        self.write_line(text, false);
    }

    fn finish(&mut self, text: &str) {
        if !self.enabled {
            return;
        }
        self.write_line(text, true);
        self.last_width = 0;
    }

    fn clear(&mut self) {
        if !self.enabled || self.last_width == 0 {
            return;
        }
        let padding = " ".repeat(self.last_width);
        let _ = write!(self.stderr, "\r{padding}\r");
        let _ = self.stderr.flush();
        self.last_width = 0;
    }

    fn write_line(&mut self, text: &str, newline: bool) {
        let width = text.chars().count();
        let padding = " ".repeat(self.last_width.saturating_sub(width));
        if newline {
            let _ = writeln!(self.stderr, "\r{text}{padding}");
        } else {
            let _ = write!(self.stderr, "\r{text}{padding}");
        }
        let _ = self.stderr.flush();
        self.last_width = width;
    }
}

impl Drop for ProgressLine {
    fn drop(&mut self) {
        self.clear();
    }
}
