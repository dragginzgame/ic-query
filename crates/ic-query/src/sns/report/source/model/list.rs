//! Module: sns::report::source::model::list
//!
//! Responsibility: source-layer deployed SNS list models.
//! Does not own: SNS-W transport, metadata conversion, or report rendering.
//! Boundary: carries resolved mainnet SNS identity data into builders.

use super::SnsSourceRequest;
use crate::sns::report::{MAINNET_SNS_WASM_CANISTER_ID, SnsHostError};
use candid::Principal;
use std::collections::BTreeSet;

///
/// MainnetSnsList
///
/// Source-layer deployed SNS list fetched from mainnet SNS-W.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsList {
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub sns_instances: Vec<MainnetSns>,
}

///
/// MainnetSns
///
/// Source-layer deployed SNS identity and optional metadata.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSns {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}

///
/// MainnetSnsCanisters
///
/// Source-layer canister ids for one deployed SNS.
///

#[expect(
    clippy::struct_field_names,
    reason = "field names identify the SNS role for each canister id"
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsCanisters {
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
    pub(in crate::sns::report) ledger_canister_id: String,
    pub(in crate::sns::report) swap_canister_id: String,
    pub(in crate::sns::report) index_canister_id: String,
}

pub(in crate::sns::report) fn validate_mainnet_sns_list(
    request: &SnsSourceRequest,
    list: &MainnetSnsList,
) -> Result<(), SnsHostError> {
    validate_provenance("network", &request.network, &list.network)?;
    validate_provenance(
        "sns_wasm_canister_id",
        MAINNET_SNS_WASM_CANISTER_ID,
        &list.sns_wasm_canister_id,
    )?;
    validate_provenance("fetched_at", &request.fetched_at, &list.fetched_at)?;
    validate_provenance("fetched_by", &request.fetched_by, &list.fetched_by)?;
    validate_provenance("source_endpoint", &request.endpoint, &list.source_endpoint)?;

    let mut roots = BTreeSet::new();
    for sns in &list.sns_instances {
        for (field, value) in [
            ("root_canister_id", sns.root_canister_id.as_str()),
            (
                "governance_canister_id",
                sns.governance_canister_id.as_str(),
            ),
            ("ledger_canister_id", sns.ledger_canister_id.as_str()),
            ("swap_canister_id", sns.swap_canister_id.as_str()),
            ("index_canister_id", sns.index_canister_id.as_str()),
        ] {
            validate_canonical_principal(field, value)?;
        }
        if !roots.insert(sns.root_canister_id.as_str()) {
            return Err(invalid_list(format!(
                "duplicate root canister id {}",
                sns.root_canister_id
            )));
        }
    }
    Ok(())
}

fn validate_provenance(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), SnsHostError> {
    if expected != actual {
        return Err(invalid_list(format!(
            "{field} is {actual:?}, expected {expected:?}"
        )));
    }
    Ok(())
}

fn validate_canonical_principal(field: &'static str, value: &str) -> Result<(), SnsHostError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_list(format!("{field} {value:?} is invalid: {error}")))?;
    if principal.to_text() != value {
        return Err(invalid_list(format!(
            "{field} {value:?} is not canonical principal text"
        )));
    }
    Ok(())
}

const fn invalid_list(reason: String) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: "SNS-W deployed SNS list",
        reason,
    }
}
