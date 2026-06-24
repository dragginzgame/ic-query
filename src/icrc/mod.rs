//! Module: icrc
//!
//! Responsibility: top-level generic ICRC ledger query commands.
//! Does not own: SNS lookup, NNS registry cache behavior, or release flow.
//! Boundary: exposes live read-only token metadata and account balance reports.

mod commands;
mod live;
mod model;
mod text;

pub use commands::run;

#[cfg(test)]
mod tests;
