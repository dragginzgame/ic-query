//! Parse and dispatch explicitly targeted neuron reads.
mod error;
mod options;
mod run;
mod spec;
pub use error::SnsCommandError;
pub use run::{command, run_matches};

#[cfg(test)]
mod tests;
