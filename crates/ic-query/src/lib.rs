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
