//! Reusable report models and helpers for Internet Computer metadata queries.
//!
//! The default feature set is empty. A dependency using
//! `default-features = false` gets the pure report DTOs, renderers, and local
//! parsing/resolution helpers that are intended to stay free of native
//! live-call and CLI dependencies.
//!
//! This is a host/CLI dependency boundary, not a `no_std` promise. No-default
//! builds are expected to compile for `wasm32-unknown-unknown` without
//! `ic-agent`, Tokio, `futures`, or `clap`, but they may still use ordinary
//! `std` types such as `String` and `Vec`.
//!
//! Enable `host` for native live-call adapters and runtime helpers. Enable
//! `cli` for the family-level command adapters used by `ic-query-cli`; `cli`
//! implies `host`.

#[cfg(feature = "host")]
mod cache_file;
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "host")]
mod duration;
#[cfg(feature = "host")]
mod hex;
#[cfg(feature = "host")]
mod ic_registry;
pub mod icrc;
pub mod nns;
#[cfg(feature = "host")]
mod output;
#[cfg(feature = "host")]
mod progress;
#[cfg(feature = "host")]
mod project;
#[cfg(feature = "host")]
mod runtime;
#[cfg(feature = "host")]
pub(crate) mod snapshot_cache;
pub mod sns;
pub mod subnet_catalog;
mod table;
#[cfg(feature = "host")]
mod text_search;
mod token_amount;
mod token_metadata_text;

#[cfg(all(test, feature = "host"))]
mod test_support;

const VERSION_TEXT: &str = concat!("icq ", env!("CARGO_PKG_VERSION"));

#[must_use]
pub const fn version_text() -> &'static str {
    VERSION_TEXT
}

#[cfg(test)]
mod tests;
