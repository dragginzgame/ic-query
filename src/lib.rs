mod cache_file;
mod cli;
mod duration;
mod hex;
mod ic_registry;
mod nns;
mod output;
mod progress;
mod project;
mod runtime;
pub(crate) mod snapshot_cache;
mod sns;
pub mod subnet_catalog;
mod table;
mod token_amount;

#[cfg(test)]
mod test_support;

use crate::cli::{
    clap::{parse_matches_or_usage, passthrough_args, string_option},
    globals::{apply_global_network, command_local_global_option, top_level_dispatch_command},
    help::{collect_args_or_print_help, usage},
};
use std::ffi::OsString;
use thiserror::Error as ThisError;

const VERSION_TEXT: &str = concat!("icq ", env!("CARGO_PKG_VERSION"));

///
/// IcQueryError
///
#[derive(Debug, ThisError)]
pub enum IcQueryError {
    #[error("{0}")]
    Usage(String),

    #[error("nns: {0}")]
    Nns(String),

    #[error("sns: {0}")]
    Sns(String),
}

/// Run the CLI from process arguments.
pub fn run_from_env() -> Result<(), IcQueryError> {
    run(std::env::args_os().skip(1))
}

/// Run the CLI from an argument iterator.
pub fn run<I>(args: I) -> Result<(), IcQueryError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help(args, usage) else {
        return Ok(());
    };
    if let Some(option) = command_local_global_option(&args) {
        return Err(IcQueryError::Usage(format!(
            "{option} is a top-level option; put it before the command\n\n{}",
            usage()
        )));
    }

    let matches = parse_matches_or_usage(top_level_dispatch_command(), args, usage)
        .map_err(IcQueryError::Usage)?;
    if matches.get_flag("version") {
        println!("{}", version_text());
        return Ok(());
    }
    let global_network = string_option(&matches, "network");

    let Some((command, subcommand_matches)) = matches.subcommand() else {
        return Err(IcQueryError::Usage(usage()));
    };
    let mut tail = passthrough_args(subcommand_matches);
    apply_global_network(command, &mut tail, global_network);
    let tail = tail.into_iter();

    match command {
        "nns" => nns::run(tail).map_err(|err| IcQueryError::Nns(err.to_string())),
        "sns" => sns::run(tail).map_err(|err| IcQueryError::Sns(err.to_string())),
        _ => unreachable!("top-level dispatch command only defines known commands"),
    }
}

#[must_use]
pub const fn version_text() -> &'static str {
    VERSION_TEXT
}

#[cfg(test)]
mod tests;
