//! Module: icrc
//!
//! Responsibility: top-level generic ICRC ledger query commands.
//! Does not own: SNS lookup, NNS registry cache behavior, or release flow.
//! Boundary: exposes live read-only token metadata, account balance, allowance,
//! index discovery, transaction history, block type, and archive reports.

mod commands;
pub mod ledger;
mod live;
mod model;
mod text;

pub use commands::run;

#[cfg(test)]
mod tests;
