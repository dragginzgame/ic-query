//! Module: sns::report::live::convert::metadata
//!
//! Responsibility: convert SNS ledger metadata and metadata errors.
//! Does not own: ledger transport, token report assembly, or rendering.
//! Boundary: maps ICRC metadata wire values into report rows and compact errors.

use crate::sns::report::SnsHostError;

/// Return a compact bounded-query error summary when the error is displayable.
pub(in crate::sns::report::live) fn bounded_query_error_summary(
    err: &SnsHostError,
) -> Option<String> {
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
        _ => None,
    }?;
    let summary = summary.trim();
    (!summary.is_empty()).then(|| summary.to_string())
}
