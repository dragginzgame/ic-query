//! Development-only command transport; Host owns command IO, groups and reaping.

use ic_host_fs::durable::create_private_bytes_with_parents;
use ic_host_process::{
    child::{CleanupPolicy, OwnedChild},
    tool::{
        ExecutionError, ExecutionEvidence, ExecutionFailure, OutputLimits, SuccessfulExit,
        ToolError, communicate_child,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    io::{self, Read},
    os::unix::{
        net::UnixStream,
        process::{CommandExt, ExitStatusExt},
    },
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const USAGE: &str = "Governance process helper\n\nCommands:\n  help\n  python <interpreter> <args...>\n  run <request-file> <result-file> <cancellation-socket>";

///
/// Request
///
/// One harness-selected command with its IO and lifetime policy.
///

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    args: Vec<String>,
    cwd: PathBuf,
    input: Option<String>,
    capture: bool,
    handoff: bool,
    stderr: String,
    timeout: f64,
}

impl Request {
    fn command(&self) -> io::Result<Command> {
        let (program, args) = self.args.split_first().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "command requires an executable",
            )
        })?;
        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(&self.cwd)
            .stdin(if self.input.is_some() {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .stdout(if self.capture {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .stderr(match self.stderr.as_str() {
                "capture" => Stdio::piped(),
                "inherit" => Stdio::inherit(),
                "discard" => Stdio::null(),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid stderr policy",
                    ));
                }
            });
        Ok(command)
    }
}

///
/// Failure
///
/// Operation failure independent of cleanup evidence and child exit status.
///

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Failure {
    Timeout,
    Cancelled,
    Io,
}

///
/// Outcome
///
/// Exact captured bytes, command status and separate cleanup diagnostics.
///

#[derive(Default, Serialize)]
struct Outcome {
    code: Option<i32>,
    signal: Option<i32>,
    failure: Option<Failure>,
    diagnostic: Option<String>,
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
    cleanup_errors: Vec<String>,
}

impl Outcome {
    fn execution_error(&mut self, error: &ExecutionError) {
        self.failure = match error.failure {
            ExecutionFailure::ExitStatus => None,
            ExecutionFailure::TimedOut => Some(Failure::Timeout),
            ExecutionFailure::Cancelled => Some(Failure::Cancelled),
            _ => Some(Failure::Io),
        };
        self.diagnostic = self.failure.as_ref().map(|_| error.to_string());
        for (stage, error) in [
            ("SIGTERM", &error.term_error),
            ("SIGKILL", &error.group_error),
            ("kill leader", &error.kill_error),
            ("reap leader", &error.wait_error),
        ] {
            if let Some(error) = error {
                self.cleanup_errors.push(format!("{stage}: {error}"));
            }
        }
    }
}

fn supervise(request: &Request, cancelled: &AtomicBool) -> io::Result<Outcome> {
    let timeout = Duration::try_from_secs_f64(request.timeout)
        .ok()
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "deadline must be positive and finite",
            )
        })?;
    let mut command = request.command()?;
    let mut child = OwnedChild::spawn_with_cleanup(
        &mut command,
        CleanupPolicy::TermThenKill {
            grace: Duration::from_secs(5),
            reap_timeout: Duration::from_secs(5),
        },
    )?;
    let started = Instant::now();
    let mut outcome = Outcome::default();
    let evidence = match communicate_child(
        &mut child,
        request.input.as_deref().map(str::as_bytes),
        // Preserve the harness's existing capture policy; admission owns report limits.
        OutputLimits {
            stdout_bytes: usize::MAX,
            stderr_bytes: usize::MAX,
            timeout,
        },
        if request.handoff {
            SuccessfulExit::Retain
        } else {
            SuccessfulExit::Cleanup
        },
        || cancelled.load(Ordering::Acquire),
    ) {
        Ok(evidence) => evidence,
        Err(ToolError::Execution(error)) => {
            outcome.execution_error(&error);
            error.evidence
        }
        Err(error) => {
            outcome.failure = Some(Failure::Io);
            outcome.diagnostic = Some(error.to_string());
            if let Err(cleanup) = child.terminate() {
                outcome.cleanup_errors.push(cleanup.to_string());
            }
            ExecutionEvidence::default()
        }
    };
    if let Some(status) = evidence.status {
        outcome.code = status.code();
        outcome.signal = status.signal();
    }
    if request.capture {
        outcome.stdout = Some(evidence.stdout);
    }
    if request.stderr == "capture" {
        outcome.stderr = Some(evidence.stderr);
    }
    if request.handoff && outcome.failure.is_none() && outcome.code == Some(0) {
        if cancelled.load(Ordering::Acquire) {
            outcome.failure = Some(Failure::Cancelled);
            outcome.diagnostic = Some("harness cancelled before handoff".into());
        } else if started.elapsed() >= timeout {
            outcome.failure = Some(Failure::Timeout);
            outcome.diagnostic = Some("deadline expired before handoff".into());
        } else if let Some(error) = [&outcome.stdout, &outcome.stderr]
            .into_iter()
            .flatten()
            .find_map(|bytes| str::from_utf8(bytes).err())
        {
            outcome.failure = Some(Failure::Io);
            outcome.diagnostic = Some(format!("invalid command text before handoff: {error}"));
        } else if let Err(error) = child.handoff() {
            outcome.failure = Some(Failure::Io);
            outcome.diagnostic = Some(error.to_string());
        }
        if outcome.failure.is_some()
            && let Err(error) = child.terminate()
        {
            outcome.cleanup_errors.push(error.to_string());
        }
    }
    Ok(outcome)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [] => println!("{USAGE}"),
        [command] if command == "help" => println!("{USAGE}"),
        [command, interpreter, args @ ..] if command == "python" => {
            return Err(Command::new(interpreter)
                .args(args)
                .env("IC_QUERY_PROCESS_TOOL", env::current_exe()?)
                .exec()
                .into());
        }
        [command, request_path, result_path, socket_path] if command == "run" => {
            let mut control = UnixStream::connect(socket_path)?;
            let request: Request = serde_json::from_slice(&std::fs::read(request_path)?)?;
            let cancelled = Arc::new(AtomicBool::new(false));
            let cancellation = Arc::clone(&cancelled);
            // The parent closes its private channel to cancel, without addressing child PIDs.
            std::thread::spawn(move || {
                let mut byte = [0];
                while let Err(error) = control.read(&mut byte) {
                    if error.kind() != io::ErrorKind::Interrupted {
                        break;
                    }
                }
                cancellation.store(true, Ordering::Release);
            });
            let outcome = supervise(&request, &cancelled).unwrap_or_else(|error| Outcome {
                failure: Some(Failure::Io),
                diagnostic: Some(error.to_string()),
                ..Outcome::default()
            });
            create_private_bytes_with_parents(
                std::path::Path::new(result_path),
                &serde_json::to_vec(&outcome)?,
            )?;
        }
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into()),
    }
    Ok(())
}
