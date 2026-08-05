use serde::{Deserialize, Serialize};

#[cfg(feature = "nns-host")]
pub(super) const NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION: u32 = 2;

///
/// NnsRegistryVersionRequest
///
/// Request for the current NNS registry version report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsRegistryVersionRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

impl NnsRegistryVersionRequest {
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
        }
    }
}

///
/// NnsRegistryVersionReport
///
/// Current NNS registry version report with source metadata.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsRegistryVersionReport {
    pub schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    /// Authenticated evidence for the certified latest version.
    pub certification: NnsRegistryCertification,
}

///
/// NnsRegistryCertification
///
/// Authenticated certificate and hash-tree evidence for the Registry version.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsRegistryCertification {
    /// Whether the certificate and version witness were authenticated.
    pub certificate_verified: bool,
    /// Certificate time in raw nanoseconds since the Unix epoch.
    pub certificate_time_nanos: u64,
    /// Certificate time formatted at UTC second precision.
    pub certificate_time: String,
    /// SHA-256 digest of the trusted DER-encoded root key.
    pub root_key_digest: String,
    /// CBOR system certificate encoded as lowercase hexadecimal.
    pub certificate_hex: String,
    /// Raw certificate length in bytes.
    pub certificate_bytes: usize,
    /// Protobuf mixed hash-tree witness encoded as lowercase hexadecimal.
    pub hash_tree_hex: String,
    /// Encoded mixed hash-tree witness length in bytes.
    pub hash_tree_bytes: usize,
}
