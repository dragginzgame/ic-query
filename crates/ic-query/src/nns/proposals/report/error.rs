//! Module: nns::proposals::report::error
//!
//! Responsibility: expose portable and native-wrapper NNS proposal failures.
//! Does not own: transport execution, cache mechanics, or process presentation.
//! Boundary: keeps canister/custom callers independent of native host dependencies.

use crate::nns::governance::{NnsGovernanceError, NnsGovernanceSourceProvenance};
#[cfg(feature = "nns-host")]
use crate::{
    HostCacheError, nns::governance::NnsGovernanceAttemptReadError, runtime::RuntimeError,
};
#[cfg(feature = "nns-host")]
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// NnsProposalError
///
/// Portable failure returned while collecting or assembling proposal reports.
///

#[derive(Debug, ThisError)]
pub enum NnsProposalError {
    /// Shared Governance request, transport, or provenance validation failed.
    #[error(transparent)]
    Governance(#[from] NnsGovernanceError),

    /// The requested page size is outside the bounded proposal contract.
    #[error("invalid NNS proposal page limit {limit}; expected 1..={maximum}")]
    InvalidLimit {
        /// Requested live-page row limit.
        limit: u32,
        /// Largest limit supported by Governance.
        maximum: u32,
    },

    /// Governance returned more proposal rows than requested.
    #[error("NNS proposal page returned {actual} rows; requested at most {requested}")]
    PageTooLarge {
        /// Number of rows returned by the source.
        actual: usize,
        /// Row ceiling supplied to the source.
        requested: u32,
    },

    /// A proposal page row did not include its required identifier.
    #[error("NNS proposal list page returned a row without a proposal id")]
    MissingProposalIdInPage,

    /// A proposal page used the reserved zero identifier.
    #[error("NNS proposal list page returned proposal id zero")]
    InvalidProposalIdInPage,

    /// A proposal page repeated an identifier.
    #[error("NNS proposal page returned duplicate proposal id {proposal_id}")]
    DuplicateProposalId {
        /// Identifier repeated within the returned page.
        proposal_id: u64,
    },

    /// A proposal did not satisfy the exclusive page cursor.
    #[error(
        "NNS proposal page returned id {proposal_id}; expected every id below {before_proposal_id}"
    )]
    ProposalCursorMismatch {
        /// Returned proposal that violates the exclusive cursor.
        proposal_id: u64,
        /// Requested exclusive upper proposal-id bound.
        before_proposal_id: u64,
    },

    /// Governance did not return the requested proposal.
    #[error("NNS proposal {proposal_id} was not found")]
    ProposalNotFound {
        /// Exact identifier requested by the caller.
        proposal_id: u64,
    },

    /// Governance returned a different proposal than requested.
    #[error("NNS proposal detail returned id {actual:?}; expected {expected}")]
    ProposalIdMismatch {
        /// Exact identifier requested by the caller.
        expected: u64,
        /// Identifier supplied in the returned row, if present.
        actual: Option<u64>,
    },

    /// A resumable collection was configured without any page capacity.
    #[error("invalid NNS proposal collection page limit 0; expected at least one page")]
    InvalidCollectionMaxPages,

    /// Serialized or caller-modified resumable collection state is inconsistent.
    #[error("invalid NNS proposal collection state: {reason}")]
    InvalidCollectionState {
        /// Deterministic state invariant that failed.
        reason: String,
    },

    /// A continuation request changed an identity fixed when collection started.
    #[error(
        "NNS proposal collection request changed {field}: received {actual}, expected {expected}"
    )]
    CollectionRequestMismatch {
        /// Fixed request field that changed.
        field: &'static str,
        /// Value retained by the collection state.
        expected: String,
        /// Value supplied by the continuation request.
        actual: String,
    },

    /// A caller attempted to advance a collection that already exhausted the API.
    #[error("NNS proposal collection is already complete after {pages_fetched} pages")]
    CollectionComplete {
        /// Pages admitted before API exhaustion.
        pages_fetched: u32,
    },

    /// A caller attempted to advance a collection after consuming its page budget.
    #[error(
        "NNS proposal collection reached its {max_pages}-page limit after {pages_fetched} pages"
    )]
    CollectionPageLimitReached {
        /// Pages admitted before the stopped advance.
        pages_fetched: u32,
        /// Configured cumulative page ceiling.
        max_pages: u32,
    },

    /// A later collection page was returned by a different collector.
    #[error("NNS proposal collection source changed: received {actual:?}, expected {expected:?}")]
    CollectionSourceChanged {
        /// Concrete source retained from the first admitted page.
        expected: NnsGovernanceSourceProvenance,
        /// Concrete source returned with the candidate page.
        actual: NnsGovernanceSourceProvenance,
    },

    /// Cumulative page or row accounting exceeded its integer representation.
    #[error("NNS proposal collection accounting overflow")]
    CollectionAccountingOverflow,
}

///
/// NnsProposalHostError
///
/// Native wrapper failure for proposal live calls, caches, and refreshes.
///

#[cfg(feature = "nns-host")]
#[derive(Debug, ThisError)]
pub enum NnsProposalHostError {
    /// Local activity input or aggregation failed.
    #[error(transparent)]
    Activity(#[from] super::NnsProposalActivityError),

    /// Portable proposal collection or validation failed.
    #[error(transparent)]
    Proposal(#[from] NnsProposalError),

    /// Confined cache IO or refresh locking failed.
    #[error(transparent)]
    Cache(#[from] HostCacheError),

    /// A snapshot's stored identity disagrees with the requested collection.
    #[error(
        "cached NNS proposal snapshot identity mismatch at {}: {field} is {actual}, expected {expected}",
        path.display()
    )]
    CacheIdentityMismatch {
        /// Snapshot file with inconsistent identity.
        path: PathBuf,
        /// Identity field that failed validation.
        field: &'static str,
        /// Value required by the requested collection.
        expected: String,
        /// Value found in the snapshot.
        actual: String,
    },

    /// Collection stopped before API exhaustion and did not replace the snapshot.
    #[error(
        "NNS proposal refresh did not publish a complete snapshot after {pages_fetched} pages and {rows_fetched} rows: {reason}"
    )]
    IncompleteRefresh {
        /// Number of pages admitted before collection stopped.
        pages_fetched: u32,
        /// Number of rows admitted before collection stopped.
        rows_fetched: usize,
        /// Failure to establish a complete sequential collection.
        reason: String,
    },

    /// Refresh requested a page size outside the supported range.
    #[error("invalid NNS proposal refresh page size {page_size}; expected 1..={max_page_size}")]
    InvalidRefreshPageSize {
        /// Rejected per-page row limit.
        page_size: u32,
        /// Largest supported per-page row limit.
        max_page_size: u32,
    },

    /// A cache-only read could not find the required complete snapshot.
    #[error("NNS proposals cache is missing at {}\n\nRun `icq nns proposal refresh` to fetch a complete snapshot.", path.display())]
    MissingProposalCache {
        /// Expected complete snapshot path.
        path: PathBuf,
    },

    /// A retained refresh-attempt sidecar failed its current contract.
    #[error("invalid NNS proposal refresh attempt at {}: {reason}", path.display())]
    InvalidRefreshAttempt {
        /// Refresh-attempt sidecar that failed validation.
        path: PathBuf,
        /// Sidecar validation diagnostic.
        reason: String,
    },

    /// A retained proposal snapshot failed its current contract.
    #[error("invalid NNS proposal cache at {}: {reason}", path.display())]
    InvalidCache {
        /// Snapshot file that failed validation.
        path: PathBuf,
        /// Snapshot validation diagnostic.
        reason: String,
    },

    /// The native runtime bridge could not execute the query.
    #[error("failed to create Tokio runtime for NNS proposal query: {0}")]
    Runtime(#[from] RuntimeError),
}

#[cfg(feature = "nns-host")]
impl From<NnsGovernanceAttemptReadError> for NnsProposalHostError {
    fn from(error: NnsGovernanceAttemptReadError) -> Self {
        match error {
            NnsGovernanceAttemptReadError::Cache(error) => Self::Cache(error),
            NnsGovernanceAttemptReadError::Invalid { path, reason } => {
                Self::InvalidRefreshAttempt { path, reason }
            }
        }
    }
}
