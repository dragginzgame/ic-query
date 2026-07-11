//! Module: runtime
//!
//! Responsibility: run async query flows from synchronous report builders.
//! Does not own: source-specific errors, command parsing, or report rendering.
//! Boundary: creates a current-thread Tokio runtime for one query flow.

use std::{future::Future, io, thread};
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

    #[error("Tokio query runtime thread panicked")]
    QueryThreadPanicked,
}

pub fn block_on_current_thread<F>(future: F) -> Result<F::Output, RuntimeError>
where
    F: Future + Send,
    F::Output: Send,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        return thread::scope(|scope| {
            scope
                .spawn(move || run_on_current_thread_runtime(future))
                .join()
                .map_err(|_| RuntimeError::QueryThreadPanicked)?
        });
    }

    run_on_current_thread_runtime(future)
}

fn run_on_current_thread_runtime<F>(future: F) -> Result<F::Output, RuntimeError>
where
    F: Future,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|source| RuntimeError::CreateTokioRuntime { source })?;
    Ok(runtime.block_on(future))
}

#[cfg(test)]
mod tests {
    use super::block_on_current_thread;

    #[test]
    fn block_on_current_thread_is_safe_inside_existing_tokio_runtime() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("outer runtime");

        let value = runtime.block_on(async {
            block_on_current_thread(async { 42_u8 }).expect("nested query runtime")
        });

        assert_eq!(value, 42);
    }
}
