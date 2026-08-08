use serde::{Deserialize, Serialize};

#[cfg(feature = "certified-subnet-catalog-host")]
pub(super) const NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION: u32 = 1;
/// Version of the complete retained certified Registry delta-report contract.
pub const NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION: u32 = 1;

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

///
/// NnsCertifiedRegistryDeltaBatchRequest
///
/// Request for one authenticated, bounded Registry delta batch.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryDeltaBatchRequest {
    /// Network identity; only mainnet `ic` is supported.
    pub network: String,
    /// Replica endpoint used for the certified query.
    pub source_endpoint: String,
    /// Last Registry version already held by the caller.
    pub requested_version: u64,
    /// Caller observation time used to validate certificate freshness.
    pub now_unix_secs: u64,
}

impl NnsCertifiedRegistryDeltaBatchRequest {
    /// Create a request for the batch immediately after `requested_version`.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        requested_version: u64,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            requested_version,
            now_unix_secs,
        }
    }
}

///
/// NnsCertifiedRegistryDeltaBatchReport
///
/// Authenticated contiguous Registry mutations returned by one bounded query.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsCertifiedRegistryDeltaBatchReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Network identity.
    pub network: String,
    /// Canonical mainnet Registry canister principal.
    pub registry_canister_id: String,
    /// Version after which deltas were requested.
    pub requested_version: u64,
    /// Latest Registry version authenticated by the same response.
    pub certified_latest_version: u64,
    /// First visible contiguous delta version, when any.
    pub first_version: Option<u64>,
    /// Last visible contiguous delta version, when any.
    pub last_version: Option<u64>,
    /// Number of visible Registry versions.
    pub version_count: usize,
    /// Number of mutations across all visible versions.
    pub mutation_count: usize,
    /// Number of preconditions across all visible versions.
    pub precondition_count: usize,
    /// Complete inline value bytes across all visible mutations.
    pub inline_value_bytes: usize,
    /// Complete reconstructed bytes for chunk-referenced values.
    pub chunk_value_bytes: usize,
    /// Complete inline and reconstructed value bytes.
    pub value_bytes: usize,
    /// Number of certified chunk references across all visible mutations.
    pub chunk_reference_count: usize,
    /// Complete bytes retained once per unique content-addressed chunk.
    pub chunk_evidence_bytes: usize,
    /// Whether later certified versions require another explicit request.
    pub more_available: bool,
    /// Caller collection time.
    pub fetched_at: String,
    /// Exact replica endpoint used by the source.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Number of Registry queries made for this batch.
    pub query_call_count: u64,
    /// Number of content-addressed `get_chunk` queries made for this batch.
    pub chunk_query_call_count: u64,
    /// Encoded certified delta response size returned by the replica.
    pub certified_response_bytes: usize,
    /// Encoded `get_chunk` response bytes returned by the replica.
    pub chunk_response_bytes: usize,
    /// Total encoded response bytes returned across every Registry query.
    pub response_bytes: usize,
    /// Resource ceilings applied while validating the batch.
    pub limits: NnsCertifiedRegistryDeltaLimits,
    /// Ordered contiguous Registry versions.
    pub versions: Vec<NnsCertifiedRegistryDeltaVersion>,
    /// Unique hash-verified chunks in canonical digest order.
    pub chunk_evidence: Vec<NnsCertifiedRegistryChunkEvidence>,
    /// Certificate and mixed-tree evidence authenticating the batch.
    pub certification: NnsRegistryCertification,
}

///
/// NnsCertifiedRegistryChunkEvidence
///
/// One unique Registry chunk retained with its content-addressed digest.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsCertifiedRegistryChunkEvidence {
    /// SHA-256 digest as exactly 64 lowercase hexadecimal characters.
    pub sha256_hex: String,
    /// Complete decoded chunk content as lowercase hexadecimal.
    pub content_hex: String,
}

///
/// NnsCertifiedRegistryDeltaLimits
///
/// Fixed resource ceilings enforced by the certified delta validator.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsCertifiedRegistryDeltaLimits {
    /// Maximum visible Registry versions in one response.
    pub max_versions: usize,
    /// Maximum total mutations in one response.
    pub max_mutations: usize,
    /// Maximum total preconditions in one response.
    pub max_preconditions: usize,
    /// Maximum bytes in one Registry key.
    pub max_key_bytes: usize,
    /// Maximum combined inline value bytes.
    pub max_inline_value_bytes: usize,
    /// Maximum chunk references across the complete batch.
    pub max_chunk_references: usize,
    /// Maximum decoded bytes in one retrieved Registry chunk.
    pub max_chunk_bytes: usize,
    /// Maximum reconstructed bytes in one Registry value.
    pub max_reconstructed_value_bytes: usize,
    /// Maximum combined inline and reconstructed value bytes.
    pub max_value_bytes: usize,
    /// Maximum encoded bytes across all `get_chunk` responses.
    pub max_chunk_response_bytes: usize,
    /// Maximum encoded bytes accepted for any single agent response body.
    pub max_response_body_bytes: usize,
}

///
/// NnsCertifiedRegistryDeltaVersion
///
/// One Registry version and its ordered atomic mutation contents.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsCertifiedRegistryDeltaVersion {
    /// Registry version authenticated by the delta-map label.
    pub version: u64,
    /// Registry-assigned mutation timestamp in nanoseconds since the Unix epoch.
    pub timestamp_nanoseconds: u64,
    /// Ordered mutations applied in this atomic version.
    pub mutations: Vec<NnsCertifiedRegistryMutation>,
    /// Preconditions attached to this atomic mutation.
    pub preconditions: Vec<NnsCertifiedRegistryPrecondition>,
}

///
/// NnsCertifiedRegistryMutation
///
/// One certified Registry mutation with raw and typed operation evidence.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsCertifiedRegistryMutation {
    /// Raw upstream protobuf mutation discriminant.
    pub mutation_type: i32,
    /// Supported meaning of the raw discriminant.
    pub mutation_kind: NnsCertifiedRegistryMutationKind,
    /// Raw Registry key bytes as lowercase hexadecimal.
    pub key_hex: String,
    /// Original certified representation of this mutation's value.
    pub value_encoding: NnsCertifiedRegistryValueEncoding,
    /// Ordered certified chunk digests as lowercase hexadecimal.
    pub chunk_sha256_hexes: Vec<String>,
    /// Complete value bytes as lowercase hexadecimal.
    ///
    /// Usually absent for deletes, but historical committed deletes may retain
    /// ignored content that replay must preserve as raw evidence.
    pub value_hex: Option<String>,
}

///
/// NnsCertifiedRegistryValueEncoding
///
/// Original value representation committed by a certified Registry mutation.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsCertifiedRegistryValueEncoding {
    /// Delete mutation with no retained value content.
    Absent,
    /// Value bytes carried directly in the certified delta response, including ignored delete content.
    Inline,
    /// Value reconstructed from certified SHA-256 chunk references, including ignored delete content.
    Chunked,
}

///
/// NnsCertifiedRegistryMutationKind
///
/// Supported native Registry mutation operations.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsCertifiedRegistryMutationKind {
    /// Insert a key that must not already exist.
    Insert,
    /// Update a key that must already exist.
    Update,
    /// Delete a key.
    Delete,
    /// Insert or update a key.
    Upsert,
}

#[cfg(feature = "certified-subnet-catalog-host")]
impl NnsCertifiedRegistryMutationKind {
    pub(super) const fn raw_type(self) -> i32 {
        match self {
            Self::Insert => 0,
            Self::Update => 1,
            Self::Delete => 2,
            Self::Upsert => 4,
        }
    }

    pub(super) const fn from_raw_type(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Insert),
            1 => Some(Self::Update),
            2 => Some(Self::Delete),
            4 => Some(Self::Upsert),
            _ => None,
        }
    }
}

///
/// NnsCertifiedRegistryPrecondition
///
/// One key-version precondition attached to a certified atomic mutation.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsCertifiedRegistryPrecondition {
    /// Raw Registry key bytes as lowercase hexadecimal.
    pub key_hex: String,
    /// Required version of the Registry key.
    pub expected_version: u64,
}
