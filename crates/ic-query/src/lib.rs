//! Feature-selected access to IC Query's independent report libraries.
//!
//! Defaults are empty. Enable `topology`, `sns`, or both. These are direct
//! re-exports: report types, query policy and implementation belong to the
//! specialist crates. This crate adds no transport, state or policy.
//!
//! Version 0.104 replaces the legacy 0.103 API. Old feature and module names
//! are not aliases; consumers must migrate explicitly. Existing specialist
//! report JSON is unchanged, but legacy retained data has no automatic migration.

/// IC infrastructure evidence for Canic decisions.
///
/// ```
/// fn same_type(report: ic_query::topology::placement::PlacementReport)
///     -> ic_query_topology::placement::PlacementReport {
///     report
/// }
/// ```
#[cfg(feature = "topology")]
pub use ic_query_topology as topology;

/// Selected ICRC balances and SNS neuron migration evidence.
///
/// The name allows future SNS capabilities; the current scope remains bounded
/// read-only migration evidence, including balances outside SNS deployments.
///
/// ```
/// fn same_type(report: ic_query::sns::balance::BalanceReport)
///     -> ic_query_sns::balance::BalanceReport {
///     report
/// }
/// ```
#[cfg(feature = "sns")]
pub use ic_query_sns as sns;
