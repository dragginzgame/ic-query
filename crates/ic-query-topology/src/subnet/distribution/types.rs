//! Distribution evidence and explicit partial outcomes.

use std::time::Duration;

use ic_platform_protocol::registry as protobuf;
use ic_platform_types::Principal;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;

use super::super::configuration::{SubnetConfigurationError, SubnetConfigurationReport};
use crate::ReadError;
use crate::node::{NodeError, NodeReport};

/// One selected subnet's version-pinned Registry relationships.
#[derive(Debug, Serialize)]
pub struct SubnetDistributionReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// Fixed admitted mainnet endpoint.
    pub endpoint: &'static str,
    /// Fixed Registry canister.
    pub source_canister: &'static str,
    /// Version-selection method; record reads use `get_value`.
    pub version_method: &'static str,
    /// Explicit nonzero version requested for every record.
    pub requested_registry_version: u64,
    /// Local completion time of the version query, in Unix seconds.
    pub version_observed_at_unix_seconds: u64,
    /// Local start time in Unix seconds.
    pub started_at_unix_seconds: u64,
    /// Monotonic duration of collection.
    pub elapsed: Duration,
    /// Platform verified query signatures; this is not consensus certification.
    pub query_signature_verified: bool,
    /// False: query results are not consensus-certified.
    pub result_certified: bool,
    /// True when the shared asynchronous deadline stopped collection.
    pub overall_deadline_reached: bool,
    /// True only when all node relationships and referenced records resolved.
    /// Empty membership is vacuously complete, not proof of an active subnet.
    pub complete: bool,
    /// Admitted membership and configuration at the requested version.
    pub configuration: SubnetConfigurationReport,
    /// One result per selected node, in canonical principal order.
    pub nodes: Vec<RecordRead<Principal, NodeReport>>,
    /// One result per distinct referenced operator.
    pub operators: Vec<RecordRead<Principal, OperatorReport>>,
    /// One result per distinct admitted lowercase data-center identity.
    pub data_centers: Vec<RecordRead<String, DataCenterReport>>,
    /// Independent joins for every selected node, including unavailable dependencies.
    pub relationships: Vec<NodeRelationships>,
    /// Counts retain selected membership as the denominator.
    pub summary: DistributionSummary,
}

/// Attempt at one exact Registry key; timings distinguish unstarted/cancelled reads.
#[derive(Debug, Serialize)]
pub struct RecordRead<K, T> {
    /// Identity used to construct this key.
    pub id: K,
    /// Exact queried key, retained even when reading failed.
    pub source_key: String,
    /// Offset from collection start; absent if never started.
    pub started_after: Option<Duration>,
    /// Completion offset; absent for cancelled or unstarted reads.
    pub finished_after: Option<Duration>,
    /// Admitted report or explicit read failure.
    pub outcome: RecordOutcome<T>,
}

/// Record-level outcomes, without claims about machine health.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RecordOutcome<T> {
    /// Fully admitted record.
    Read {
        /// Selected fields and mutation evidence.
        report: T,
    },
    /// Failed read or admission; typed cause remains available in Rust.
    Failed {
        /// JSON preserves the category and raw Registry rejection fields.
        error: DistributionError,
    },
    /// Started and cancelled by the shared deadline.
    OverallDeadline,
    /// The shared deadline prevented this read from starting.
    NotStarted,
}

impl<T> RecordOutcome<T> {
    pub(super) fn report(&self) -> Option<&T> {
        match self {
            Self::Read { report } => Some(report),
            _ => None,
        }
    }
}

/// Mutation evidence, distinct from the globally requested version.
#[derive(Debug, Serialize)]
pub struct RecordEvidence {
    /// Record method, always `get_value`.
    pub source_method: &'static str,
    /// Mutation version, admitted at or below the requested version.
    pub record_version: u64,
    /// Raw Registry mutation timestamp in nanoseconds.
    pub record_timestamp_nanoseconds: u64,
    /// Local completion time in Unix seconds.
    pub observed_at_unix_seconds: u64,
}

/// Bounded provider reference; missing/invalid bytes do not erase location evidence.
#[derive(Debug, Serialize)]
pub struct ProviderReference {
    /// Raw bytes when within the 29-byte principal budget; None means oversized.
    pub raw: Option<Vec<u8>>,
    /// Source byte count, including oversized refusals.
    pub byte_length: usize,
    /// Nonempty admitted principal; None is missing or invalid, never management.
    pub identity: Option<Principal>,
}

/// Bounded data-center reference, preserving raw text separately from key identity.
#[derive(Debug, Serialize)]
pub struct DataCenterReference {
    /// Raw text within 256 UTF-8 bytes; None means oversized, not absent.
    pub raw: Option<String>,
    /// Source byte count, including oversized refusals.
    pub byte_length: usize,
    /// Lowercase key identity; missing/invalid references are not queried.
    pub identity: Option<String>,
}

/// Operator relationships from one admitted Registry record.
#[derive(Debug, Serialize)]
pub struct OperatorReport {
    /// Raw embedded operator bytes; empty is allowed, populated must match the key.
    pub registered_operator_bytes: Vec<u8>,
    /// Independently admitted provider reference.
    pub provider: ProviderReference,
    /// Independently admitted location reference.
    pub data_center: DataCenterReference,
    /// Mutation and observation evidence.
    pub evidence: RecordEvidence,
}

/// Registered data-center declarations, not independently verified geography.
#[derive(Debug, Serialize)]
pub struct DataCenterReport {
    /// Raw source ID, admitted against the lowercase queried identity.
    pub id: String,
    /// Raw region text, at most 256 UTF-8 bytes; may be empty.
    pub region: String,
    /// Raw owner text, at most 256 UTF-8 bytes; not a provider principal.
    pub owner: String,
    /// Mutation and observation evidence.
    pub evidence: RecordEvidence,
}

/// A resolved relationship or explicit reason that the dependency is unavailable.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Relationship<T> {
    /// Admitted relationship.
    Resolved {
        /// Principal or lowercase data-center key identity.
        value: T,
    },
    /// A required earlier read or reference did not resolve.
    Unavailable {
        /// Stable category; original read failures remain in their record rows.
        reason: UnavailableReason,
    },
}

/// Why a selected node's provider or location could not be resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    /// The selected node could not be read/admitted.
    NodeRecord,
    /// Its operator could not be read/admitted.
    OperatorRecord,
    /// An operator record contained an empty reference.
    MissingReference,
    /// A populated reference failed admission.
    InvalidReference,
    /// A valid data-center reference could not be read/admitted.
    DataCenterRecord,
}

/// Independent relationship coverage for one selected member.
#[derive(Debug, Serialize)]
pub struct NodeRelationships {
    /// Selected membership identity.
    pub node_id: Principal,
    /// Provider joined through the admitted operator.
    pub provider: Relationship<Principal>,
    /// Known data-center reference, even when its record is unavailable.
    pub data_center_reference: Relationship<String>,
    /// Data center with successfully read and admitted details.
    pub location: Relationship<String>,
}

/// Selected-node count for one resolved identity or raw declaration.
#[derive(Debug, Serialize)]
pub struct GroupCount<T> {
    /// Group identity; text values remain raw in JSON.
    pub value: T,
    /// Number of selected nodes, not distinct operators or total network nodes.
    pub nodes: usize,
}

/// Distribution counts with explicit independent coverage denominators.
#[derive(Debug, Serialize)]
pub struct DistributionSummary {
    /// Full admitted membership, including every unresolved node.
    pub selected_nodes: usize,
    /// Selected nodes with admitted provider identities.
    pub provider_resolved: usize,
    /// Selected nodes without an admitted provider.
    pub provider_unknown: usize,
    /// Selected nodes with an admitted data-center reference.
    pub data_center_references_resolved: usize,
    /// Selected nodes with successfully read location details.
    pub location_resolved: usize,
    /// Selected nodes without successfully read location details.
    pub location_unknown: usize,
    /// Counts by provider, in canonical principal order.
    pub providers: Vec<GroupCount<Principal>>,
    /// Counts by successfully read data center, in lowercase identity order.
    pub data_centers: Vec<GroupCount<String>>,
    /// Counts by nonempty raw region declaration; empty regions remain unknown.
    pub regions: Vec<GroupCount<String>>,
    /// Selected nodes without a nonempty region declaration.
    pub region_unknown: usize,
    /// Counts by nonempty raw owner declaration; not verified provider ownership.
    pub owners: Vec<GroupCount<String>>,
    /// Selected nodes without a nonempty owner declaration.
    pub owner_unknown: usize,
}

/// Collection or record-admission failure; source causes remain typed in Rust.
#[derive(Debug, Error)]
pub enum DistributionError {
    /// Mainnet authentication, transport, deadline or clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// External bounded codec failure.
    #[error(transparent)]
    Codec(#[from] protobuf::Error),
    /// Fixed Registry identity conversion failed.
    #[error("invalid Registry identity: {0}")]
    SourceIdentity(String),
    /// Configuration admission failed before fan-out.
    #[error(transparent)]
    Configuration(#[from] SubnetConfigurationError),
    /// Node admission failed.
    #[error(transparent)]
    Node(#[from] NodeError),
    /// Zero selects latest rather than a fixed version.
    #[error("Registry version zero cannot establish a fixed version")]
    ZeroVersion,
    /// The overall budget expired before configuration was admitted.
    #[error("distribution deadline expired before configuration admission")]
    InitialDeadline,
    /// Membership exceeds the selected-subnet scope.
    #[error("distribution membership exceeds the 128-node limit")]
    NodeLimit,
    /// The response mutation version is newer than the requested version.
    #[error("record version {returned} exceeds requested Registry version {requested}")]
    VersionSkew {
        /// Requested global version.
        requested: u64,
        /// Refused record mutation version.
        returned: u64,
    },
    /// Registry application rejection; raw text is not displayed by Error.
    #[error("Registry rejected distribution record (code {code})")]
    RegistryRejected {
        /// Raw numeric source code.
        code: i32,
        /// Raw source reason, requiring escaped presentation.
        reason: String,
        /// Raw source key bytes.
        key: Vec<u8>,
    },
    /// Absent inline content, distinct from a present empty proto3 message.
    #[error("Registry returned no inline record")]
    MissingContent,
    /// Populated operator identity or returned data-center ID did not match its key.
    #[error("Registry record identity does not match its queried key")]
    IdentityMismatch,
    /// Data-center details exceeded their per-field byte budgets.
    #[error("data-center text exceeds the 256-byte limit")]
    TextLimit,
}

impl Serialize for DistributionError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let kind = match self {
            Self::Read(ReadError::Transport(_)) => "transport_or_verification",
            Self::Read(ReadError::Deadline) => "read_deadline",
            Self::Read(ReadError::Clock(_) | ReadError::ClockRange) => "clock",
            Self::Codec(_) => "codec",
            Self::SourceIdentity(_) => "source_identity",
            Self::Configuration(_) => "configuration_admission",
            Self::Node(_) => "node_admission",
            Self::ZeroVersion => "zero_version",
            Self::InitialDeadline => "initial_deadline",
            Self::NodeLimit => "node_limit",
            Self::VersionSkew { .. } => "version_skew",
            Self::RegistryRejected { .. } => "registry_rejected",
            Self::MissingContent => "missing_content",
            Self::IdentityMismatch => "identity_mismatch",
            Self::TextLimit => "text_limit",
        };
        let count = match self {
            Self::RegistryRejected { .. } => 4,
            Self::VersionSkew { .. } => 3,
            _ => 1,
        };
        let mut fields = serializer.serialize_struct("DistributionError", count)?;
        fields.serialize_field("kind", kind)?;
        if let Self::RegistryRejected { code, reason, key } = self {
            fields.serialize_field("code", code)?;
            fields.serialize_field("reason", reason)?;
            fields.serialize_field("key", key)?;
        }
        if let Self::VersionSkew {
            requested,
            returned,
        } = self
        {
            fields.serialize_field("requested", requested)?;
            fields.serialize_field("returned", returned)?;
        }
        fields.end()
    }
}
