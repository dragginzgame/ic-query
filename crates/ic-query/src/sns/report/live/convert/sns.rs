//! Module: sns::report::live::convert::sns
//!
//! Responsibility: convert deployed SNS and root metadata wire values.
//! Does not own: SNS-W transport, metadata fetch scheduling, or report rendering.
//! Boundary: maps live SNS-W/root responses into source-layer SNS identity models.

use super::common::clean_optional_text;
use crate::sns::report::{
    MainnetSnsCanisters, MainnetSnsMetadata, SnsHostError,
    live::types::{DeployedSns, GetMetadataResponse},
};
use candid::Principal;

/// Convert one SNS-W deployed SNS entry into required canister id strings.
pub(in crate::sns::report::live) fn mainnet_sns_canisters_from_deployed_sns(
    sns: DeployedSns,
) -> Result<MainnetSnsCanisters, SnsHostError> {
    Ok(MainnetSnsCanisters {
        root_canister_id: required_principal_text(sns.root_canister_id, "root_canister_id")?,
        governance_canister_id: required_principal_text(
            sns.governance_canister_id,
            "governance_canister_id",
        )?,
        ledger_canister_id: required_principal_text(sns.ledger_canister_id, "ledger_canister_id")?,
        swap_canister_id: required_principal_text(sns.swap_canister_id, "swap_canister_id")?,
        index_canister_id: required_principal_text(sns.index_canister_id, "index_canister_id")?,
    })
}

/// Convert one Governance metadata response into a keyed source result.
pub(in crate::sns::report::live) fn mainnet_sns_metadata_from_response(
    root_canister_id: String,
    metadata: GetMetadataResponse,
    metadata_error: Option<String>,
) -> MainnetSnsMetadata {
    MainnetSnsMetadata {
        root_canister_id,
        name: clean_optional_text(metadata.name),
        description: clean_optional_text(metadata.description),
        url: clean_optional_text(metadata.url),
        metadata_error,
    }
}

fn required_principal_text(
    principal: Option<Principal>,
    field: &'static str,
) -> Result<String, SnsHostError> {
    principal
        .map(|principal| principal.to_text())
        .ok_or_else(|| SnsHostError::InvalidPrincipal {
            field,
            reason: "missing principal".to_string(),
        })
}
