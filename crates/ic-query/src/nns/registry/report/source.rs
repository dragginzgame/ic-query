use super::{error::NnsRegistryHostError, model::NnsRegistryCertification};
use crate::{
    ic_registry::{MainnetRegistryVersion, fetch_mainnet_registry_version},
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
};

///
/// NnsRegistryVersionData
///
/// Source-layer NNS registry version result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsRegistryVersionData {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    /// Authenticated evidence for the certified latest version.
    pub certification: NnsRegistryCertification,
}

impl From<MainnetRegistryVersion> for NnsRegistryVersionData {
    fn from(version: MainnetRegistryVersion) -> Self {
        Self {
            network: version.network,
            registry_canister_id: version.registry_canister_id,
            registry_version: version.registry_version,
            fetched_at: version.fetched_at,
            fetched_by: version.fetched_by,
            source_endpoint: version.source_endpoint,
            certification: NnsRegistryCertification {
                certificate_verified: version.certification.certificate_verified,
                certificate_time_nanos: version.certification.certificate_time_nanos,
                certificate_time: version.certification.certificate_time,
                root_key_digest: version.certification.root_key_digest,
                certificate_hex: version.certification.certificate_hex,
                certificate_bytes: version.certification.certificate_bytes,
                hash_tree_hex: version.certification.hash_tree_hex,
                hash_tree_bytes: version.certification.hash_tree_bytes,
            },
        }
    }
}

///
/// NnsRegistrySource
///
/// Source contract for fetching NNS registry version data.
///

pub trait NnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError>;
}

impl NnsRegistrySource for LiveNnsSource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
        let fetch_request = mainnet_registry_fetch_request(request, |network| {
            NnsRegistryHostError::UnsupportedNetwork { network }
        })?;
        Ok(fetch_mainnet_registry_version(&fetch_request)?.into())
    }
}
