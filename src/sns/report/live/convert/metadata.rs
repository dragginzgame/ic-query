//! Module: sns::report::live::convert::metadata
//!
//! Responsibility: convert SNS ledger metadata and metadata errors.
//! Does not own: ledger transport, token report assembly, or rendering.
//! Boundary: maps ICRC metadata wire values into report rows and compact errors.

use crate::sns::report::{
    SNS_TOKEN_LOGO_METADATA_KEY, SnsHostError, SnsTokenMetadataRow, hex_bytes,
    live::types::{GetIndexPrincipalError, IcrcMetadataValue},
};
use serde_json::Value as JsonValue;

/// Convert one ICRC metadata key/value pair into a report row.
pub(in crate::sns::report) fn metadata_row(
    key: String,
    value: IcrcMetadataValue,
) -> SnsTokenMetadataRow {
    if key == SNS_TOKEN_LOGO_METADATA_KEY {
        return SnsTokenMetadataRow {
            key,
            value_type: "bool".to_string(),
            value: JsonValue::Bool(metadata_value_is_present(&value)),
        };
    }

    let (value_type, value) = match value {
        IcrcMetadataValue::Nat(value) => ("nat", value.to_string()),
        IcrcMetadataValue::Int(value) => ("int", value.to_string()),
        IcrcMetadataValue::Text(value) => ("text", value),
        IcrcMetadataValue::Blob(value) => ("blob", hex_bytes(&value)),
    };
    SnsTokenMetadataRow {
        key,
        value_type: value_type.to_string(),
        value: JsonValue::String(value),
    }
}

/// Convert an index-principal discovery error into human-facing text.
pub(in crate::sns::report::live) fn index_principal_error_text(
    error: GetIndexPrincipalError,
) -> String {
    match error {
        GetIndexPrincipalError::IndexPrincipalNotSet => "index principal not set".to_string(),
        GetIndexPrincipalError::GenericError {
            error_code,
            description,
        } => format!("generic error {error_code}: {description}"),
    }
}

/// Return a compact metadata-fetch error summary when the error is displayable.
pub(in crate::sns::report::live) fn metadata_error_summary(err: &SnsHostError) -> Option<String> {
    match err {
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
        | SnsHostError::InvalidPrincipal { .. }
        | SnsHostError::UnknownSnsId { .. }
        | SnsHostError::UnknownSnsRoot { .. }
        | SnsHostError::InvalidLookup { .. }
        | SnsHostError::MissingNeuronsCache { .. }
        | SnsHostError::MissingNeuronsCacheForId { .. }
        | SnsHostError::MissingProposalsCache { .. }
        | SnsHostError::ReadCache { .. }
        | SnsHostError::ParseCache { .. }
        | SnsHostError::SerializeCache { .. }
        | SnsHostError::UnsupportedCacheSchemaVersion { .. }
        | SnsHostError::CacheNetworkMismatch { .. }
        | SnsHostError::Cache(_)
        | SnsHostError::IncompleteRefresh { .. }
        | SnsHostError::MissingCacheRoot
        | SnsHostError::UnsupportedProposalView { .. } => None,
    }
}

fn metadata_value_is_present(value: &IcrcMetadataValue) -> bool {
    match value {
        IcrcMetadataValue::Text(value) => !value.trim().is_empty(),
        IcrcMetadataValue::Blob(value) => !value.is_empty(),
        IcrcMetadataValue::Nat(_) | IcrcMetadataValue::Int(_) => true,
    }
}
