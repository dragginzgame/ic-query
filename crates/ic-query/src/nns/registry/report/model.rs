use serde::{Deserialize, Serialize};

///
/// NnsRegistryInventoryProvenance
///
/// Registry authority, version and acquisition metadata shared by NNS inventories.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsRegistryInventoryProvenance {
    /// Network whose Registry inventory was collected.
    pub network: String,
    /// Registry principal supplying the inventory records.
    pub registry_canister_id: String,
    /// Registry version used for the complete collection.
    pub registry_version: u64,
    /// UTC collection timestamp with second precision.
    pub fetched_at: String,
    /// Explicit replica endpoint used for collection.
    pub source_endpoint: String,
    /// Collector identity recorded at acquisition.
    pub fetched_by: String,
}

#[cfg(feature = "nns-host")]
pub(super) const NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION: u32 = 1;
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
    /// Shared record, flattened in JSON.
    #[serde(flatten)]
    pub provenance: NnsRegistryInventoryProvenance,
    /// Authenticated evidence for the certified latest version.
    pub certification: NnsRegistryCertification,
}

///
/// NnsRegistryCertification
///
/// Authenticated certificate and hash-tree evidence for the Registry version.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsRegistryCertification {
    /// Whether the certificate and version witness were authenticated.
    pub certificate_verified: bool,
    /// Certificate time in raw nanoseconds since the Unix epoch.
    pub certificate_time_nanos: u64,
    /// SHA-256 digest of the trusted DER-encoded root key.
    pub root_key_digest: String,
    /// CBOR system certificate encoded as lowercase hexadecimal.
    pub certificate_hex: String,
    /// Protobuf mixed hash-tree witness encoded as lowercase hexadecimal.
    pub hash_tree_hex: String,
}

///
/// NnsRegistryVersionData
///
/// Source-layer NNS registry version result.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsRegistryVersionData {
    /// Shared record, flattened in JSON.
    #[serde(flatten)]
    pub provenance: NnsRegistryInventoryProvenance,
    /// Authenticated evidence for the certified latest version.
    pub certification: NnsRegistryCertification,
}

impl NnsRegistryCertification {
    /// Return the decoded length of canonical certificate hex evidence.
    #[must_use]
    pub const fn certificate_bytes(&self) -> usize {
        self.certificate_hex.len() / 2
    }

    /// Return the decoded length of canonical mixed-tree hex evidence.
    #[must_use]
    pub const fn hash_tree_bytes(&self) -> usize {
        self.hash_tree_hex.len() / 2
    }
}
