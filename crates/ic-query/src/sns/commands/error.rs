//! Module: sns::commands::error
//!
//! Responsibility: define errors surfaced by SNS command parsing and runtime.
//! Does not own: report-layer host errors or text rendering.
//! Boundary: converts command setup failures into user-facing CLI errors.

use crate::{cli::common::CurrentUnixSecsError, sns::report::SnsHostError};
use std::io;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum SnsCommandError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    Host(#[from] SnsHostError),

    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
