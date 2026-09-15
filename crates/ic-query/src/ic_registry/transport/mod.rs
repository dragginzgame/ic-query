mod acquisition;
#[cfg(feature = "certified-subnet-catalog-host")]
mod certified;
#[cfg(feature = "certified-subnet-catalog-host")]
mod certified_delta;
mod chunk;
mod codec;
mod key_family;
mod query;
mod value;
mod version;

pub(super) use crate::hex::hex_bytes;
#[cfg(feature = "certified-subnet-catalog-host")]
pub(super) use certified::get_certified_latest_version;
#[cfg(feature = "certified-subnet-catalog-host")]
pub use certified_delta::authenticate_certified_registry_delta_witness;
#[cfg(feature = "certified-subnet-catalog-host")]
pub(super) use certified_delta::get_certified_changes_since;
#[cfg(feature = "certified-subnet-catalog-host")]
pub use certified_delta::{
    MAX_CERTIFIED_DELTA_INLINE_VALUE_BYTES, MAX_CERTIFIED_DELTA_KEY_BYTES,
    MAX_CERTIFIED_DELTA_MUTATIONS, MAX_CERTIFIED_DELTA_PRECONDITIONS, MAX_CERTIFIED_DELTA_VERSIONS,
};
#[cfg(feature = "certified-subnet-catalog-host")]
pub use chunk::{
    MAX_CERTIFIED_DELTA_VALUE_BYTES, MAX_REGISTRY_CHUNK_BYTES, MAX_REGISTRY_CHUNK_REFERENCES,
    MAX_REGISTRY_CHUNK_RESPONSE_BYTES, MAX_REGISTRY_RECONSTRUCTED_VALUE_BYTES,
};
pub(super) use codec::decode_message;
pub(in crate::ic_registry) use key_family::get_registry_key_family_counted;
#[cfg(feature = "nns-topology-host")]
pub(super) use value::get_registry_value;
pub(super) use value::get_registry_versioned_value_counted;
#[cfg(all(test, feature = "nns-host"))]
pub(super) use value::registry_value_content_from_response;
#[cfg(feature = "nns-topology-host")]
pub(super) use version::get_latest_version;
pub(super) use version::get_latest_version_counted;

pub use acquisition::RegistryAcquisition;
pub use acquisition::{SubnetCatalogProgress, SubnetCatalogProgressPhase};

use std::sync::atomic::{AtomicU64, Ordering};

///
/// RegistryQueryCounter
///
/// Shared low-level query-attempt counter for one concurrent Registry collection.
///

#[derive(Default)]
pub(super) struct RegistryQueryCounter {
    calls: AtomicU64,
    endpoint: String,
    acquisition: Option<std::sync::Arc<RegistryAcquisition>>,
}

impl RegistryQueryCounter {
    pub(super) const fn with_acquisition(
        endpoint: String,
        acquisition: std::sync::Arc<RegistryAcquisition>,
    ) -> Self {
        Self {
            calls: AtomicU64::new(0),
            endpoint,
            acquisition: Some(acquisition),
        }
    }

    pub(super) fn emit(&self, phase: SubnetCatalogProgressPhase) {
        if let Some(callback) = self
            .acquisition
            .as_ref()
            .and_then(|context| context.progress.as_ref())
        {
            callback(SubnetCatalogProgress {
                endpoint: self.endpoint.clone(),
                query_call_count: self.call_count(),
                phase,
            });
        }
    }

    pub(super) fn record_call(&self) {
        self.calls.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn call_count(&self) -> u64 {
        self.calls.load(Ordering::Relaxed)
    }
}
