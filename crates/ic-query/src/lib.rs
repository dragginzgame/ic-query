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
//! Enable `host` for native live-call adapters and runtime helpers. CLI parsing
//! and process IO belong to the separate `ic-query-cli` crate.

#[cfg(feature = "host")]
mod agent;
#[cfg(feature = "host")]
mod cache_file;
pub mod duration;
#[cfg(feature = "host")]
mod freshness;
mod hex;
pub mod ic;
#[cfg(feature = "host")]
mod ic_registry;
pub mod icrc;
#[cfg(feature = "host")]
mod network;
pub mod nns;
#[cfg(feature = "host")]
mod progress;
#[cfg(feature = "host")]
mod runtime;
#[cfg(feature = "host")]
pub(crate) mod snapshot_cache;
pub mod sns;
pub mod subnet_catalog;

#[cfg(feature = "host")]
pub use cache_file::{CacheFileError, HostCacheError};
mod table;
#[cfg(feature = "host")]
mod text_search;
mod text_value;
mod token_amount;
mod token_metadata_text;

#[cfg(all(test, feature = "host"))]
mod test_support;

#[cfg(test)]
mod tests;

#[cfg(feature = "host")]
pub use progress::{QueryProgress, QueryProgressEvent, QueryProgressState};
