//! Module: cache
//!
//! Responsibility: expose local user-level cache inspection commands.
//! Does not own: cache discovery, report construction, or cache mutation.
//! Boundary: resolves the CLI cache root and writes library status reports.

use crate::{
    cli::common::{
        COLLECTION_MODE_CACHE_ONLY, CurrentUnixSecsError, collection_help, current_unix_secs,
        json_arg, output_format, write_text_or_json,
    },
    storage::{CacheRootError, cache_root},
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::cache::{
    CacheStatusError, CacheStatusRequest, build_cache_status_report, cache_status_report_text,
};
use std::io;
use thiserror::Error as ThisError;

const STATUS_HELP_AFTER: &str = "\
Examples:
  icq cache status
  icq cache status --json";

///
/// CacheCommandError
///
/// Errors surfaced while inspecting the local user-level cache root.
///

#[derive(Debug, ThisError)]
pub enum CacheCommandError {
    /// Resolving the user-level cache root failed.
    #[error(transparent)]
    CacheRoot(#[from] CacheRootError),
    /// Local cache traversal failed.
    #[error(transparent)]
    Status(#[from] CacheStatusError),
    /// The process clock could not supply an observation timestamp.
    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),
    /// Writing selected report output failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// JSON serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq cache")
        .about("Inspect the local ic-query cache")
        .subcommand(
            ClapCommand::new("status")
                .bin_name("icq cache status")
                .about("Show every known complete cache and its age policy")
                .arg(json_arg())
                .after_help(collection_help(
                    COLLECTION_MODE_CACHE_ONLY,
                    STATUS_HELP_AFTER,
                )),
        )
}

pub fn run_matches(matches: &ArgMatches) -> Result<(), CacheCommandError> {
    match matches.subcommand() {
        Some(("status", matches)) => {
            let report = build_cache_status_report(&CacheStatusRequest::new(
                cache_root()?,
                current_unix_secs()?,
            ))?;
            write_text_or_json(output_format(matches), &report, cache_status_report_text)
        }
        _ => unreachable!("clap requires a known cache subcommand"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::render_help;
    use std::ffi::OsString;

    #[test]
    fn help_describes_local_only_cache_status() {
        let help = render_help(command());
        assert!(help.contains("status"));
        let status = command().find_subcommand("status").expect("status").clone();
        let help = render_help(status);
        assert!(help.contains("--json"));
        assert!(help.contains(COLLECTION_MODE_CACHE_ONLY));
    }

    #[test]
    fn help_does_not_inspect_the_filesystem() {
        for args in [&["cache", "--help"][..], &["cache", "status", "--help"]] {
            assert!(crate::run(args.iter().map(OsString::from)).is_ok());
        }
    }
}
