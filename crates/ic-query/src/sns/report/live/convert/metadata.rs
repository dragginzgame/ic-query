//! Module: sns::report::live::convert::metadata
//!
//! Responsibility: convert SNS ledger metadata and metadata errors.
//! Does not own: ledger transport, token report assembly, or rendering.
//! Boundary: maps ICRC metadata wire values into report rows and compact errors.

use crate::sns::report::SnsHostError;

/// Return a compact metadata-fetch error summary when the error is displayable.
pub(in crate::sns::report::live) fn metadata_error_summary(err: &SnsHostError) -> Option<String> {
    let summary = match err {
        SnsHostError::AgentCall { method, reason } => Some(format!("{method}: {reason}")),
        SnsHostError::CandidEncode { message, reason } => {
            Some(format!("encode {message}: {reason}"))
        }
        SnsHostError::CandidDecode { message, reason } => {
            Some(format!("decode {message}: {reason}"))
        }
        SnsHostError::GovernanceError {
            method,
            error_type,
            message,
        } => Some(format!("{method} governance error {error_type}: {message}")),
        SnsHostError::MissingGovernanceResult { method } => {
            Some(format!("{method}: missing governance result"))
        }
        SnsHostError::UnsupportedNetwork { .. }
        | SnsHostError::Runtime(_)
        | SnsHostError::AgentBuild { .. }
        | SnsHostError::AgentUpdateCall { .. }
        | SnsHostError::InvalidPrincipal { .. }
        | SnsHostError::InvalidSourceData { .. }
        | SnsHostError::MissingRunningSnsVersion { .. }
        | SnsHostError::MissingProposalId
        | SnsHostError::MissingNeuronId
        | SnsHostError::InvalidNeuronId
        | SnsHostError::InvalidNeuronIdText { .. }
        | SnsHostError::MissingNeuronPermissionPrincipal { .. }
        | SnsHostError::UnstableRewardCheckpoint { .. }
        | SnsHostError::InvalidRewardCheckpointCeiling { .. }
        | SnsHostError::InvalidRewardCheckpointPageCap { .. }
        | SnsHostError::IncompleteRewardCheckpoint { .. }
        | SnsHostError::RewardCheckpointArithmetic { .. }
        | SnsHostError::RewardCheckpointClock { .. }
        | SnsHostError::ReadRewardCheckpoint { .. }
        | SnsHostError::ParseRewardCheckpoint { .. }
        | SnsHostError::UnknownSnsId { .. }
        | SnsHostError::UnknownSnsRoot { .. }
        | SnsHostError::InvalidLookup { .. }
        | SnsHostError::InvalidMetricsTimeWindow { .. }
        | SnsHostError::AmbiguousCacheId { .. }
        | SnsHostError::AmbiguousRefreshAttemptId { .. }
        | SnsHostError::MissingCatalogCache { .. }
        | SnsHostError::MissingNeuronsCache { .. }
        | SnsHostError::MissingNeuronsCacheForId { .. }
        | SnsHostError::MissingProposalsCache { .. }
        | SnsHostError::ReadCache { .. }
        | SnsHostError::ParseCache { .. }
        | SnsHostError::InvalidRefreshAttempt { .. }
        | SnsHostError::InvalidCache { .. }
        | SnsHostError::SerializeCache { .. }
        | SnsHostError::UnsupportedCacheSchemaVersion { .. }
        | SnsHostError::CacheNetworkMismatch { .. }
        | SnsHostError::CacheIdentityMismatch { .. }
        | SnsHostError::Cache(_)
        | SnsHostError::IncompleteRefresh { .. }
        | SnsHostError::InvalidRefreshPageSize { .. }
        | SnsHostError::MissingCacheRoot
        | SnsHostError::UnsupportedProposalView { .. }
        | SnsHostError::DuplicateCanisterId { .. } => None,
    }?;
    let summary = summary.trim();
    (!summary.is_empty()).then(|| summary.to_string())
}
