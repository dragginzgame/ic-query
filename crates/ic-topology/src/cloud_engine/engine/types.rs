//! Selected-engine report semantics, independent of external wire records.

use crate::ReadError;
use ic_platform_protocol::{candid as decoder, cloud_engine::EncodeError};
use ic_platform_types::Principal;
use serde::Serialize;
use std::{fmt, str::FromStr, time::Duration};
use thiserror::Error;

/// Opaque nonempty engine ID, at most 256 UTF-8 bytes; never normalized.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct EngineId(String);

impl EngineId {
    /// Borrow the original identifier without trimming or normalization.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for EngineId {
    type Err = InvalidEngineId;
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if raw.is_empty() || raw.len() > 256 {
            return Err(InvalidEngineId { bytes: raw.len() });
        }
        Ok(Self(raw.to_owned()))
    }
}

impl fmt::Display for EngineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Invalid IDs are rejected before any Agent construction or network IO.
#[derive(Debug, Error)]
#[error("engine ID must contain 1–256 UTF-8 bytes (received {bytes})")]
pub struct InvalidEngineId {
    /// Original byte count; raw user text is not interpolated into diagnostics.
    pub bytes: usize,
}

/// One selected public engine lookup, including successful omission.
#[derive(Debug, Serialize)]
pub struct EngineReport {
    /// Query-owned schema; remains 1 before 1.0.
    pub schema: u8,
    /// Exact ID requested, retained even when no view was returned.
    pub requested_engine_id: EngineId,
    /// Local start time in Unix seconds.
    pub started_at_unix_seconds: u64,
    /// Monotonic collection duration.
    pub elapsed: Duration,
    /// False: a control-plane view is not an atomic IC inventory.
    pub atomic_snapshot: bool,
    /// Successful absence or admitted engine record.
    pub outcome: EngineOutcome,
    /// Source assurance and local response-completion time.
    pub evidence: EngineEvidence,
}

/// Missing IDs and hidden reservations cannot be distinguished anonymously.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum EngineOutcome {
    /// No view returned; not proof of nonexistence or deletion.
    NotReturned,
    /// An exact-ID record, without asserting an active or live engine.
    Read {
        /// Selected raw facts and an explicit membership comparison.
        record: Box<EngineRecord>,
    },
}

/// Control-plane relationships, keeping two different node-list sources apart.
#[derive(Debug, Serialize)]
pub struct EngineRecord {
    /// Optional subnet binding; absence does not establish its cause.
    pub subnet_id: Option<Principal>,
    /// Optional engine-operator canister; not the Registry's node operator.
    pub operator_canister_id: Option<Principal>,
    /// Engine-record snapshot, preserving raw acquisition order.
    pub record_node_ids: Vec<Principal>,
    /// Raw signed deletion timestamp, with None distinct from zero or negative.
    pub deleted_at_unix_nanoseconds: Option<i64>,
    /// The source's Registry copy; absence is not empty membership.
    pub registry_copy_subnet: Option<RegistryCopySubnet>,
    /// Set differences, unavailable when the nested copy is absent.
    pub membership: MembershipComparison,
}

/// Embedded Registry-copy facts; no version or copy timestamp is exposed.
#[derive(Debug, Serialize)]
pub struct RegistryCopySubnet {
    /// Node identities in source order; not verified liveness.
    pub node_ids: Vec<Principal>,
    /// Raw configured text; empty denotes the source's standard engine version.
    pub configured_replica_version: String,
    /// Raw registered halt flag, not an uptime measurement.
    pub halted: bool,
    /// Raw optional limit in bytes; not free or measured state size.
    pub maximum_state_size_bytes: Option<u64>,
}

/// Comparison uses validated principal sets, not source vector order.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MembershipComparison {
    /// No embedded copy was returned; do not compare against an invented empty set.
    Unavailable,
    /// Both lists were present and admitted; equality does not prove freshness.
    Compared {
        /// Engine-record nodes absent from the embedded copy, in principal order.
        record_only: Vec<Principal>,
        /// Embedded-copy nodes absent from the engine record, in principal order.
        registry_copy_only: Vec<Principal>,
    },
}

/// Evidence from the one fixed, anonymous mainnet query.
#[derive(Debug, Serialize)]
pub struct EngineEvidence {
    /// Fixed admitted HTTPS endpoint.
    pub endpoint: &'static str,
    /// Exact engine-registry canister principal.
    pub source_canister: &'static str,
    /// Exact public query method.
    pub source_method: &'static str,
    /// Local response-completion time, not the Registry copy's timestamp.
    pub observed_at_unix_seconds: u64,
    /// Agent verified the query signature.
    pub query_signature_verified: bool,
    /// False: query signatures do not certify result consensus.
    pub result_certified: bool,
}

/// Query-owned refusal of a decoded selected-engine view.
#[derive(Debug, Error)]
pub enum EngineRefusal {
    /// A selected field or collection exceeds its report retention bound.
    #[error("engine {field} exceeds {maximum}: {actual}")]
    Limit {
        /// Bound being checked.
        field: &'static str,
        /// Observed byte or item count.
        actual: usize,
        /// Allowed maximum.
        maximum: usize,
    },
    /// Returned identity differs from the exact request.
    #[error("returned engine ID does not match the requested ID")]
    WrongEngine,
    /// Management/empty principal cannot identify the selected infrastructure.
    #[error("engine {field} contains an empty identity")]
    EmptyIdentity {
        /// Identity field being checked.
        field: &'static str,
    },
    /// One source list repeats a node identity.
    #[error("engine {field} repeats node {node_id}")]
    DuplicateNode {
        /// Which independent list was refused.
        field: &'static str,
        /// Repeated canonical identity.
        node_id: Principal,
    },
    /// The documented nested-subnet relationship lacks its subnet binding.
    #[error("engine contains Registry-copy subnet data without a subnet ID")]
    MissingSubnetId,
}

/// Original typed errors; failures do not fabricate an empty engine report.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Transport, verified-query policy, deadline or clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// Bounded request encoding failure.
    #[error("engine request encoding failed: {0:?}")]
    Encode(EncodeError),
    /// Bounded external decoding failure.
    #[error("engine response decoding failed: {0:?}")]
    Decode(decoder::Error),
    /// Query-owned record admission failure.
    #[error(transparent)]
    Admission(#[from] EngineRefusal),
}
