use serde::Serialize;

///
/// MainnetRegistryCertification
///
/// Authenticated certificate and hash-tree evidence for a Registry version.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetRegistryCertification {
    /// Whether the built-in source authenticated the certificate and version witness.
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

///
/// MainnetRegistryVersion
///
/// Current mainnet registry version and its source metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetRegistryVersion {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    /// Authenticated evidence for the certified latest version.
    pub certification: MainnetRegistryCertification,
}
