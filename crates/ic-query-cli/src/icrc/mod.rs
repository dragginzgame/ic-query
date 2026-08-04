//! ICRC command-line parsing and dispatch.

mod commands;
#[cfg(test)]
mod tests;

use crate::cli::common::CurrentUnixSecsError;
pub use commands::{command, run_matches};
use std::io;
use thiserror::Error as ThisError;

///
/// IcrcCommandError
///
/// Error returned by ICRC command parsing, dispatch, and output.
///

#[derive(Debug, ThisError)]
pub enum IcrcCommandError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),

    #[error(transparent)]
    Query(#[from] ic_query::icrc::IcrcError),

    #[error(transparent)]
    Analytics(#[from] ic_query::ic::IcHostError),

    #[error(transparent)]
    AccountTransaction(#[from] ic_query::icrc::IcrcAccountTransactionError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
