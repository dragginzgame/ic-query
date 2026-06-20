//! Module: cli::common
//!
//! Responsibility: shared CLI output format and clock helpers.
//! Does not own: command parsing, report construction, or command-specific rendering.
//! Boundary: writes text/JSON reports and builds common CLI arguments.

use crate::{
    cli::clap::value_arg,
    output::{write_pretty_json, write_text},
};
use clap::ValueEnum;
use serde::Serialize;
use std::{
    io,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};
use thiserror::Error as ThisError;

pub const FORMAT_ARG: &str = "format";
pub const SOURCE_ENDPOINT_ARG: &str = "source-endpoint";

const DEFAULT_FORMAT: &str = "text";

///
/// OutputFormat
///
/// User-selected CLI output format for report rendering.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

///
/// CurrentUnixSecsError
///
/// Error returned when the system clock cannot be represented as Unix seconds.
///

#[derive(Debug, ThisError)]
pub enum CurrentUnixSecsError {
    #[error("system clock before unix epoch: {source}")]
    BeforeUnixEpoch { source: SystemTimeError },
}

pub fn write_text_or_json<T, E>(
    format: OutputFormat,
    report: &T,
    render_text: impl FnOnce(&T) -> String,
) -> Result<(), E>
where
    T: Serialize,
    E: From<io::Error> + From<serde_json::Error>,
{
    match format {
        OutputFormat::Text => {
            let text = render_text(report);
            write_text::<E>(&text)
        }
        OutputFormat::Json => write_pretty_json(report),
    }
}

pub fn write_text_or_json_verbose<T, E>(
    format: OutputFormat,
    report: &T,
    verbose: bool,
    render_text: impl FnOnce(&T) -> String,
    render_verbose_text: impl FnOnce(&T) -> String,
) -> Result<(), E>
where
    T: Serialize,
    E: From<io::Error> + From<serde_json::Error>,
{
    write_text_or_json(format, report, |report| {
        if verbose {
            render_verbose_text(report)
        } else {
            render_text(report)
        }
    })
}

pub fn current_unix_secs() -> Result<u64, CurrentUnixSecsError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|source| CurrentUnixSecsError::BeforeUnixEpoch { source })
}

pub fn format_arg() -> clap::Arg {
    value_arg(FORMAT_ARG)
        .long(FORMAT_ARG)
        .value_name("text|json")
        .default_value(DEFAULT_FORMAT)
        .value_parser(clap::value_parser!(OutputFormat))
        .help("Output format; defaults to text")
}

pub fn source_endpoint_arg(default_source_endpoint: &'static str) -> clap::Arg {
    value_arg(SOURCE_ENDPOINT_ARG)
        .long(SOURCE_ENDPOINT_ARG)
        .value_name("url")
        .default_value(default_source_endpoint)
}
