//! Read-only ICRC balances and selected SNS neuron evidence for migration consumers.
//!
//! Explicit ledger/governance targets work independently of topology. Reports
//! preserve partial observations; they neither execute migrations nor determine
//! destination entitlements. Query signatures do not certify application state.

#[cfg(test)]
mod tests;

pub mod balance;
mod mainnet;
pub mod neuron;
mod observation;

pub use observation::{Failure, FailureKind, Observation, Outcome};
