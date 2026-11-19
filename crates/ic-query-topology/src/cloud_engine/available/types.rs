//! Available-node observations, traversal coverage and typed stop reasons.

use crate::ReadError;
use ic_platform_protocol::{
    candid as decoder,
    cloud_engine::{EncodeError, EngineError, NodeType},
};
use ic_platform_types::Principal;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::time::Duration;
use thiserror::Error;

/// Public new-engine eligibility observed through bounded sequential pages.
#[derive(Debug, Serialize)]
pub struct AvailableNodesReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// Caller-selected maximum admitted pages; always 1–100.
    pub page_limit: u16,
    /// Maximum admitted nodes for this request, equal to page_limit times 100.
    pub node_limit: usize,
    /// Local collection start in Unix seconds.
    pub started_at_unix_seconds: u64,
    /// Monotonic elapsed collection time.
    pub elapsed: Duration,
    /// Fixed mainnet endpoint.
    pub endpoint: &'static str,
    /// Fixed cloud-engine canister principal.
    pub source_canister: &'static str,
    /// Selected page method; status has its own method evidence.
    pub source_method: &'static str,
    /// Always absent: only public new-engine eligibility is requested.
    pub rejoining: Option<String>,
    /// Platform verified signatures for every successfully received observation.
    pub query_signature_verified: bool,
    /// Always false: verified queries do not establish consensus-certified results.
    pub result_certified: bool,
    /// False even for equal Registry versions: availability changes between reads.
    pub atomic_snapshot: bool,
    /// Always false; exhausted pagination does not prove a point-in-time census.
    pub point_in_time_inventory_complete: bool,
    /// True only after an admitted page has no continuation cursor.
    pub pagination_exhausted: bool,
    /// Independently observed copy progress, not the version pinned by this report.
    pub registry_copy: RegistryCopyObservation,
    /// Admitted pages in acquisition order, preserving source row order.
    pub pages: Vec<AvailablePage>,
    /// Why traversal stopped; failed pages never discard earlier admitted pages.
    pub stop: TraversalStop,
}

/// Independent Registry-copy progress reported by the engine canister.
#[derive(Debug, Serialize)]
pub struct RegistryCopyObservation {
    /// Exact status method.
    pub source_method: &'static str,
    /// Local response-completion time in Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Copy version reported by the source.
    pub version: u64,
    /// Most recently observed Registry latest version, not necessarily current.
    pub latest: u64,
    /// Raw source caught-up flag; not proof of mainnet freshness.
    pub caught_up: bool,
}

/// One admitted page and its request cursor, without pretending to be a snapshot.
#[derive(Debug, Serialize)]
pub struct AvailablePage {
    /// Cursor requested for this page; None means the first page.
    pub start: Option<Principal>,
    /// Source-provided next-page cursor, retained even at a traversal limit.
    pub next: Option<Principal>,
    /// Source-reported Registry-copy version for this page.
    pub registry_version: u64,
    /// Local response-completion time in Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Selected source rows; unique within this report, not measured capacity.
    pub nodes: Vec<AvailableNode>,
}

/// Source-listed node eligible for a new engine at this page's observation.
#[derive(Debug, Serialize)]
pub struct AvailableNode {
    /// Node principal.
    pub node_id: Principal,
    /// Raw external marketplace class, not a hardware-performance measurement.
    pub node_type: NodeType,
    /// Registered operator principal.
    pub operator_id: Principal,
    /// Registered provider principal.
    pub provider_id: Principal,
    /// Raw optional data-center reference, including present-empty.
    pub data_center: Option<String>,
    /// Raw source flag meaning the record carries a chip ID; not attestation.
    pub sev_enabled: bool,
}

/// Traversal terminal state, independent of point-in-time completeness.
#[derive(Debug, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum TraversalStop {
    /// Last admitted page has no continuation cursor.
    Exhausted,
    /// Selected page budget admitted with a continuation remaining.
    PageLimit,
    /// Selected node ceiling admitted with a continuation remaining.
    NodeLimit,
    /// A page could not be admitted or read; previous pages remain available.
    Failed {
        /// Requested cursor; None for a failed first page.
        start: Option<Principal>,
        /// Response-completion time when available, even for rejected pages.
        observed_at_unix_seconds: Option<u64>,
        /// Source version when a decoded page was available but refused.
        observed_registry_version: Option<u64>,
        /// Original typed source or admission failure.
        error: AvailableError,
    },
    /// Overall 60-second budget stopped or prevented the next page read.
    OverallDeadline {
        /// Cursor for the unfinished or unstarted page.
        start: Option<Principal>,
    },
}

/// Why an observed page cannot enter the report's admitted inventory.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum PageRefusal {
    /// More than 100 nodes in one page or the selected report ceiling.
    #[error("available-node {field} exceeds {maximum}: {actual}")]
    Limit {
        /// Count being bounded.
        field: &'static str,
        /// Observed count.
        actual: usize,
        /// Accepted maximum.
        maximum: usize,
    },
    /// Same node appears twice within or across pages.
    #[error("available-node response repeats node {node_id}")]
    DuplicateNode {
        /// Repeated identity.
        node_id: Principal,
    },
    /// A continuation cursor was previously returned, so traversal would cycle.
    #[error("available-node response repeats continuation cursor {cursor}")]
    RepeatedCursor {
        /// Repeated cursor identity.
        cursor: Principal,
    },
    /// Page version differs from the first admitted page's version.
    #[error("available-node Registry version changed from {expected} to {observed}")]
    VersionChanged {
        /// First admitted page's version.
        expected: u64,
        /// Refused page's version, including decreases.
        observed: u64,
    },
    /// Data-center reference exceeds its 256 UTF-8 byte retention bound.
    #[error("available-node data-center reference exceeds 256 bytes: {bytes}")]
    DataCenter {
        /// Node whose field was refused.
        node_id: Principal,
        /// Original UTF-8 byte length; contents are not retained or truncated.
        bytes: usize,
    },
    /// Empty management principal is not a node/operator/provider identity.
    #[error("available-node record has an empty {field} identity")]
    EmptyIdentity {
        /// Identity field refused.
        field: &'static str,
    },
}

/// Initial collection error or a retained failure of one page.
#[derive(Debug, Error)]
pub enum AvailableError {
    /// Page budget is outside 1–100; refused before any network IO.
    #[error("available-node page budget must be between 1 and 100: {requested}")]
    InvalidPageLimit {
        /// Refused caller value.
        requested: u16,
    },
    /// Transport, authentication, leaf deadline or local clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// Bounded request encoding error, preserved as its original typed value.
    #[error("available-node request encoding failed: {0:?}")]
    Encode(EncodeError),
    /// Bounded response codec error, preserved as its original typed value.
    #[error("available-node response decoding failed: {0:?}")]
    Decode(decoder::Error),
    /// Application rejection; raw payload is retained in Rust/JSON, not Display.
    #[error("available-node source rejected the request")]
    Application(EngineError),
    /// Explicit `ok null` response; distinct from an empty exhausted page.
    #[error("available-node Registry copy is not ready")]
    CopyNotReady,
    /// Decoded page failed report admission.
    #[error(transparent)]
    Admission(#[from] PageRefusal),
    /// The status read consumed the overall budget before a report was acquired.
    #[error("available-node collection exceeded its 60-second deadline")]
    OverallDeadline,
}

impl From<EncodeError> for AvailableError {
    fn from(error: EncodeError) -> Self {
        Self::Encode(error)
    }
}

impl From<decoder::Error> for AvailableError {
    fn from(error: decoder::Error) -> Self {
        Self::Decode(error)
    }
}

impl Serialize for AvailableError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let category = match self {
            Self::InvalidPageLimit { .. } => "invalid_page_limit",
            Self::Read(ReadError::Transport(_)) => "transport",
            Self::Read(ReadError::Deadline) => "read_deadline",
            Self::Read(ReadError::Clock(_) | ReadError::ClockRange) => "clock",
            Self::Encode(_) => "encode",
            Self::Decode(_) => "decode",
            Self::Application(_) => "application",
            Self::CopyNotReady => "copy_not_ready",
            Self::Admission(_) => "admission",
            Self::OverallDeadline => "overall_deadline",
        };
        let mut state = serializer.serialize_struct("AvailableError", 3)?;
        state.serialize_field("category", category)?;
        state.serialize_field(
            "application",
            &match self {
                Self::Application(error) => Some(error),
                _ => None,
            },
        )?;
        state.serialize_field(
            "admission",
            &match self {
                Self::Admission(error) => Some(error),
                _ => None,
            },
        )?;

        state.end()
    }
}
