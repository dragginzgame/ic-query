use crate::{
    cli::clap::value_arg,
    output::{write_pretty_json, write_text},
};
use clap::ValueEnum;
use serde::Serialize;
use std::{
    io,
    time::{SystemTime, UNIX_EPOCH},
};

pub const FORMAT_ARG: &str = "format";
pub const SOURCE_ENDPOINT_ARG: &str = "source-endpoint";

const DEFAULT_FORMAT: &str = "text";

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
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

pub fn current_unix_secs() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|err| err.to_string())
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
