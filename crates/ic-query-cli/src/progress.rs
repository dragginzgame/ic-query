//! Module: progress
//!
//! Responsibility: render library progress events for the `icq` process.
//! Does not own: refresh orchestration, cache policy, or report construction.
//! Boundary: owns terminal detection and stderr output for query progress.

use ic_host_text::progress::ProgressLine as ProgressFrames;
use ic_query::{QueryProgress, QueryProgressEvent, QueryProgressState};
use std::io::{self, IsTerminal, Write};

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
                    "{component} cache refresh required at {}; calling {source_endpoint} to refresh/create cache",
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

///
/// ProgressLine
///
/// Same-line stderr writer used by the CLI progress sink.
///

#[derive(Debug)]
struct ProgressLine {
    enabled: bool,
    frames: ProgressFrames,
    stderr: io::Stderr,
}

impl ProgressLine {
    fn stderr() -> Self {
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

impl Drop for ProgressLine {
    fn drop(&mut self) {
        self.clear();
    }
}
