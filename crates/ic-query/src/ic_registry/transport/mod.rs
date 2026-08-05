mod chunk;
mod codec;
mod value;
mod version;

use std::sync::atomic::{AtomicU64, Ordering};

///
/// RegistryQueryCounter
///
/// Shared low-level query-attempt counter for one concurrent Registry collection.
///

#[derive(Default)]
pub(super) struct RegistryQueryCounter(AtomicU64);

impl RegistryQueryCounter {
    pub(super) fn record_call(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn call_count(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

pub(super) use crate::hex::hex_bytes;
#[cfg(all(test, feature = "nns-host"))]
pub(super) use chunk::{append_validated_chunk, sha256_digest};
pub(super) use codec::decode_message;
#[cfg(feature = "nns-topology-host")]
pub(super) use value::get_registry_value;
pub(super) use value::get_registry_value_counted;
#[cfg(all(test, feature = "nns-host"))]
pub(super) use value::registry_value_content_from_response;
#[cfg(feature = "nns-topology-host")]
pub(super) use version::get_latest_version;
pub(super) use version::get_latest_version_counted;
