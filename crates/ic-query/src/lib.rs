//! Reusable report models and helpers for Internet Computer metadata queries.
//!
//! The default feature set is empty. A dependency using
//! `default-features = false` gets the pure report DTOs, renderers, and local
//! parsing/resolution helpers that are intended to stay free of native
//! live-call dependencies.
//!
//! This is a host dependency boundary, not a `no_std` promise. No-default
//! builds are expected to compile for `wasm32-unknown-unknown` without
//! `ic-agent`, Reqwest, Tokio, or `futures`, but they may still use ordinary
//! `std` types such as `String` and `Vec`.
//!
//! Enable `dashboard-host` for official Dashboard live/cache APIs,
//! `icrc-host` for native ICRC ledger/index APIs, `subnet-catalog-host` for the
//! focused live/cache Subnet catalog API, `nns-topology-host` for that API plus
//! exact-version joined NNS Subnet topology, or `host` for every native
//! live-call adapter and runtime helper. CLI parsing and process IO belong to
//! the separate `ic-query-cli` crate.

#[cfg(any(feature = "icrc-host", feature = "subnet-catalog-host"))]
mod agent;
pub mod cache;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host"
))]
mod cache_file;
#[cfg(feature = "icrc-host")]
mod certification;
pub mod duration;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host"
))]
mod freshness;
mod hex;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host"
))]
mod http_endpoint;
mod human_quantity;
pub mod ic;
#[cfg(feature = "subnet-catalog-host")]
mod ic_registry;
pub mod icrc;
#[cfg(any(feature = "dashboard-host", feature = "subnet-catalog-host"))]
mod network;
pub mod nns;
#[cfg(any(feature = "dashboard-host", feature = "icrc-host"))]
mod progress;
pub mod report;
#[cfg(feature = "host")]
mod report_sort;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host"
))]
mod runtime;
#[cfg(any(feature = "dashboard-host", feature = "icrc-host"))]
pub(crate) mod snapshot_cache;
pub mod sns;
pub mod subnet_catalog;
pub mod system;

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host"
))]
pub use cache_file::{CacheFileError, HostCacheError};
mod table;
#[cfg(feature = "host")]
mod text_search;
mod text_value;
mod token_amount;
mod token_metadata_text;

#[cfg(all(
    test,
    any(
        feature = "dashboard-host",
        feature = "icrc-host",
        feature = "subnet-catalog-host"
    )
))]
mod test_support;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "dashboard-host", feature = "icrc-host"))]
pub use progress::{QueryProgress, QueryProgressEvent, QueryProgressState};
