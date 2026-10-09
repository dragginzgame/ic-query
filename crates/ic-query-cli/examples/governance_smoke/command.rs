//! Command IO and interruption policy for the development harness.

use crate::{Result, invalid};
use ic_host_process::{
    child::{CleanupPolicy, OwnedChild},
    tool::{
        ExecutionEvidence, ExecutionFailure, OutputLimits, SuccessfulExit, ToolError,
        communicate_child,
    },
};
use std::{
    error::Error,
    fmt, io,
    process::{Command, Stdio},
    sync::atomic::{AtomicI32, Ordering},
    time::{Duration, Instant},
};

static INTERRUPTED: AtomicI32 = AtomicI32::new(0);

extern "C" fn record_signal(signal: libc::c_int) {
    // AtomicI32 is lock-free on the supported Linux and macOS architectures.
    // Subsequent signals leave the first interruption's cleanup running.
    let _ = INTERRUPTED.compare_exchange(0, signal, Ordering::Relaxed, Ordering::Relaxed);
}

pub fn interruption() -> Option<i32> {
    match INTERRUPTED.load(Ordering::Relaxed) {
        0 => None,
        signal => Some(signal),
    }
}

pub fn check_interruption() -> Result<()> {
    if let Some(signal) = interruption() {
        return Err(invalid(&format!("interrupted by {}", signal_name(signal))));
    }
    Ok(())
}

pub const fn signal_name(signal: i32) -> &'static str {
    match signal {
        libc::SIGINT => "SIGINT",
        libc::SIGTERM => "SIGTERM",
        _ => "unknown signal",
    }
}

///
/// SignalGuard
///
/// Restores the caller's handlers after the synchronous harness exits.
///

pub struct SignalGuard([(i32, libc::sighandler_t); 2]);

impl SignalGuard {
    pub fn install() -> io::Result<Self> {
        let mut previous = [
            (libc::SIGINT, libc::SIG_DFL),
            (libc::SIGTERM, libc::SIG_DFL),
        ];
        for index in 0..previous.len() {
            // The handler only records a lock-free atomic; no allocation or IO.
            let handler = unsafe {
                libc::signal(
                    previous[index].0,
                    record_signal as *const () as libc::sighandler_t,
                )
            };
            if handler == libc::SIG_ERR {
                let error = io::Error::last_os_error();
                for &(signal, old) in &previous[..index] {
                    unsafe { libc::signal(signal, old) };
                }
                return Err(error);
            }
            previous[index].1 = handler;
        }
        Ok(Self(previous))
    }
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        for &(signal, handler) in &self.0 {
            unsafe { libc::signal(signal, handler) };
        }
    }
}

///
/// StderrMode
///
/// The child command's stderr routing, independent of captured stdout.
///

#[derive(Clone, Copy, Default)]
pub enum StderrMode {
    #[default]
    Inherit,
    Capture,
    Discard,
}

///
/// CommandOptions
///
/// Caller-selected command IO, deadline and successful background ownership.
///

pub struct CommandOptions<'a> {
    pub capture: bool,
    pub stderr: StderrMode,
    pub timeout: Duration,
    pub handoff: bool,
    pub input: Option<&'a [u8]>,
    pub cleanup: bool,
}

impl Default for CommandOptions<'_> {
    fn default() -> Self {
        Self {
            capture: true,
            stderr: StderrMode::Inherit,
            timeout: Duration::from_secs(600),
            handoff: false,
            input: None,
            cleanup: false,
        }
    }
}

///
/// CommandFailure
///
/// Original operation failure with captured bytes and separate cleanup evidence.
///

#[derive(Debug)]
pub struct CommandFailure {
    pub source: Box<dyn Error>,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>,
    pub cleanup_errors: Vec<String>,
    pub exited: bool,
    pub interrupted: Option<i32>,
}

impl fmt::Display for CommandFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.source.fmt(formatter)
    }
}

impl Error for CommandFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

pub fn run(command: &mut Command, options: &CommandOptions<'_>) -> Result<String> {
    supervise(command, options, || {
        !options.cleanup && interruption().is_some()
    })
}

pub fn supervise(
    command: &mut Command,
    options: &CommandOptions<'_>,
    cancelled: impl Fn() -> bool,
) -> Result<String> {
    command
        .stdin(if options.input.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        })
        .stdout(if options.capture {
            Stdio::piped()
        } else {
            Stdio::inherit()
        })
        .stderr(match options.stderr {
            StderrMode::Inherit => Stdio::inherit(),
            StderrMode::Capture => Stdio::piped(),
            StderrMode::Discard => Stdio::null(),
        });
    let mut child = OwnedChild::spawn_with_cleanup(
        command,
        CleanupPolicy::TermThenKill {
            grace: Duration::from_secs(5),
            reap_timeout: Duration::from_secs(5),
        },
    )?;
    let started = Instant::now();
    let result = communicate_child(
        &mut child,
        options.input,
        OutputLimits {
            stdout_bytes: usize::MAX,
            stderr_bytes: usize::MAX,
            timeout: options.timeout,
        },
        if options.handoff {
            SuccessfulExit::Retain
        } else {
            SuccessfulExit::Cleanup
        },
        &cancelled,
    );
    let mut cleanup_errors = Vec::new();
    let (evidence, mut failure, exited) = match result {
        Ok(evidence) => (evidence, None, false),
        Err(ToolError::Execution(mut error)) => {
            for (stage, error) in [
                ("SIGTERM", &error.term_error),
                ("SIGKILL", &error.group_error),
                ("kill leader", &error.kill_error),
                ("reap leader", &error.wait_error),
            ] {
                if let Some(error) = error {
                    cleanup_errors.push(format!("{stage}: {error}"));
                }
            }
            let exited = matches!(error.failure, ExecutionFailure::ExitStatus);
            {
                let evidence = std::mem::take(&mut error.evidence);
                error.evidence.status = evidence.status;
                (evidence, Some(Box::new(error) as Box<dyn Error>), exited)
            }
        }
        Err(error) => {
            if let Err(error) = child.terminate() {
                cleanup_errors.push(error.to_string());
            }
            (
                ExecutionEvidence::default(),
                Some(Box::new(error) as Box<dyn Error>),
                false,
            )
        }
    };
    let text = if failure.is_none() {
        let admitted = admit_output(&evidence, options, &mut child, started, &cancelled);
        match admitted {
            Ok(text) => Some(text),
            Err(error) => {
                failure = Some(error);
                None
            }
        }
    } else {
        None
    };
    if let Some(source) = failure {
        if options.handoff
            && let Err(error) = child.terminate()
        {
            cleanup_errors.push(error.to_string());
        }
        return Err(Box::new(CommandFailure {
            source,
            stdout: options.capture.then_some(evidence.stdout),
            stderr: matches!(options.stderr, StderrMode::Capture).then_some(evidence.stderr),
            cleanup_errors,
            exited,
            interrupted: (!options.cleanup).then(interruption).flatten(),
        }));
    }
    Ok(text.unwrap_or_default())
}

fn admit_output(
    evidence: &ExecutionEvidence,
    options: &CommandOptions<'_>,
    child: &mut OwnedChild,
    started: Instant,
    cancelled: impl Fn() -> bool,
) -> Result<String> {
    if matches!(options.stderr, StderrMode::Capture) {
        str::from_utf8(&evidence.stderr)?;
    }
    let text = if options.capture {
        str::from_utf8(&evidence.stdout)?
    } else {
        ""
    };
    if options.handoff {
        if cancelled() {
            return Err(invalid("harness cancelled before handoff"));
        }
        if started.elapsed() >= options.timeout {
            return Err(
                io::Error::new(io::ErrorKind::TimedOut, "deadline expired before handoff").into(),
            );
        }
        child.handoff()?;
    }
    Ok(text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_string())
}
