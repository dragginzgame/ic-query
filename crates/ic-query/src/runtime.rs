//! Module: runtime
//!
//! Responsibility: run async query flows from synchronous report builders.
//! Does not own: source-specific errors, command parsing, or report rendering.
//! Boundary: creates a current-thread Tokio runtime for one query flow.

use std::{future::Future, io};
use thiserror::Error as ThisError;

///
/// RuntimeError
///
/// Error returned while creating the local Tokio runtime.
///

#[derive(Debug, ThisError)]
pub enum RuntimeError {
    #[error("failed to create Tokio current-thread runtime: {source}")]
    CreateTokioRuntime { source: io::Error },
}

pub fn block_on_current_thread<F>(future: F) -> Result<F::Output, RuntimeError>
where
    F: Future,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|source| RuntimeError::CreateTokioRuntime { source })?;
    Ok(runtime.block_on(future))
}
