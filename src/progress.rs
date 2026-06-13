use std::io::{self, IsTerminal, Write};

#[derive(Debug)]
pub struct ProgressLine {
    enabled: bool,
    last_width: usize,
    stderr: io::Stderr,
}

impl ProgressLine {
    #[must_use]
    pub fn stderr() -> Self {
        Self::with_enabled(io::stderr().is_terminal())
    }

    #[must_use]
    pub fn with_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            last_width: 0,
            stderr: io::stderr(),
        }
    }

    pub fn update(&mut self, text: &str) {
        if !self.enabled {
            return;
        }
        self.write_line(text, false);
    }

    pub fn finish(&mut self, text: &str) {
        if !self.enabled {
            return;
        }
        self.write_line(text, true);
        self.last_width = 0;
    }

    pub fn clear(&mut self) {
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
