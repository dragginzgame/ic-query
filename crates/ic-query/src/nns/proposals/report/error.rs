//! Module: nns::proposals::report::error
//!
//! Responsibility: expose portable and native-wrapper NNS proposal failures.
//! Does not own: transport execution, cache mechanics, or process presentation.
//! Boundary: keeps canister/custom callers independent of native host dependencies.

use crate::nns::governance::NnsGovernanceError;
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
    InvalidLimit { limit: u32, maximum: u32 },

    /// Governance returned more proposal rows than requested.
    #[error("NNS proposal page returned {actual} rows; requested at most {requested}")]
    PageTooLarge { actual: usize, requested: u32 },

    /// A proposal page row did not include its required identifier.
    #[error("NNS proposal list page returned a row without a proposal id")]
    MissingProposalIdInPage,

    /// A proposal page repeated an identifier.
    #[error("NNS proposal page returned duplicate proposal id {proposal_id}")]
    DuplicateProposalId { proposal_id: u64 },

    /// A proposal did not satisfy the exclusive page cursor.
    #[error(
        "NNS proposal page returned id {proposal_id}; expected every id below {before_proposal_id}"
    )]
    ProposalCursorMismatch {
        proposal_id: u64,
        before_proposal_id: u64,
    },

    /// Governance did not return the requested proposal.
    #[error("NNS proposal {proposal_id} was not found")]
    ProposalNotFound { proposal_id: u64 },

    /// Governance returned a different proposal than requested.
    #[error("NNS proposal detail returned id {actual:?}; expected {expected}")]
    ProposalIdMismatch { expected: u64, actual: Option<u64> },
}

///
/// NnsProposalHostError
///
/// Native wrapper failure for proposal live calls, caches, and refreshes.
///

#[cfg(feature = "nns-host")]
#[derive(Debug, ThisError)]
pub enum NnsProposalHostError {
    /// Portable proposal collection or validation failed.
    #[error(transparent)]
    Proposal(#[from] NnsProposalError),

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error(
        "cached NNS proposal snapshot identity mismatch at {}: {field} is {actual}, expected {expected}",
        path.display()
    )]
    CacheIdentityMismatch {
        path: PathBuf,
        field: &'static str,
        expected: String,
        actual: String,
    },

    #[error(
        "NNS proposal refresh did not publish a complete snapshot after {pages_fetched} pages and {rows_fetched} rows: {reason}"
    )]
    IncompleteRefresh {
        pages_fetched: u32,
        rows_fetched: usize,
        reason: String,
    },

    #[error("invalid NNS proposal refresh page size {page_size}; expected 1..={max_page_size}")]
    InvalidRefreshPageSize { page_size: u32, max_page_size: u32 },

    #[error("NNS proposals cache is missing at {}\n\nRun `icq nns proposal refresh` to fetch a complete snapshot.", path.display())]
    MissingProposalCache { path: PathBuf },

    #[error("invalid NNS proposal refresh attempt at {}: {reason}", path.display())]
    InvalidRefreshAttempt { path: PathBuf, reason: String },

    #[error("invalid NNS proposal cache at {}: {reason}", path.display())]
    InvalidCache { path: PathBuf, reason: String },

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
