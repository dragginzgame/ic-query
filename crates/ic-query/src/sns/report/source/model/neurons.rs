//! Module: sns::report::source::model::neurons
//!
//! Responsibility: source-layer SNS neuron models.
//! Does not own: governance transport, cache storage, sorting, or rendering.
//! Boundary: carries live neuron rows and pagination cursors into builders.

use super::validation::SnsSourceValidator;
use crate::{
    hex::{is_canonical_lowercase_hex, is_lowercase_hex},
    sns::report::{SnsHostError, SnsNeuronDetail, SnsNeuronRow, sns_neuron_permission_name},
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::{CandidType, Deserialize, Principal};
use std::collections::HashSet;

const SNS_NEURON_ID_HEX_LENGTH: usize = 64;
const NEURON_DETAIL_CAPABILITY: &str = "SNS neuron detail";

///
/// MainnetSnsNeurons
///
/// Source-layer bounded SNS neuron listing.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsNeurons {
    pub neurons: Vec<SnsNeuronRow>,
}

///
/// MainnetSnsNeuron
///
/// Source-layer exact SNS neuron detail result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsNeuron {
    /// Full native neuron detail returned by the source.
    pub detail: SnsNeuronDetail,
}

///
/// MainnetSnsNeuronPage
///
/// Source-layer SNS neuron page used by complete snapshot refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsNeuronPage {
    pub neurons: Vec<SnsNeuronRow>,
    pub last_cursor: Option<SnsNeuronId>,
}

///
/// SnsNeuronId
///
/// Candid-compatible SNS neuron pagination cursor.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SnsNeuronId {
    pub id: Vec<u8>,
}

/// Validate one bounded neuron result returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_neurons(
    neurons: &MainnetSnsNeurons,
    requested_limit: u32,
) -> Result<(), SnsHostError> {
    validate_sns_neuron_source_rows(&neurons.neurons, requested_limit, "SNS neurons")
}

/// Validate one neuron page returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_neuron_page(
    page: &MainnetSnsNeuronPage,
    requested_limit: u32,
) -> Result<(), SnsHostError> {
    validate_sns_neuron_source_rows(&page.neurons, requested_limit, "SNS neuron page")
}

fn validate_sns_neuron_source_rows(
    neurons: &[SnsNeuronRow],
    requested_limit: u32,
    capability: &'static str,
) -> Result<(), SnsHostError> {
    let validator = SnsSourceValidator::new(capability);
    if neurons.len() > requested_limit as usize {
        return Err(validator.invalid(format!(
            "returned {} rows for requested limit {requested_limit}",
            neurons.len()
        )));
    }
    validate_sns_neuron_rows(neurons).map_err(|reason| validator.invalid(reason))
}

/// Validate canonical row fields and neuron-id uniqueness within one row collection.
pub(in crate::sns::report) fn validate_sns_neuron_rows(
    neurons: &[SnsNeuronRow],
) -> Result<(), String> {
    let mut neuron_ids = HashSet::new();
    for neuron in neurons {
        validate_sns_neuron_row(neuron)?;
        if !neuron_ids.insert(neuron.neuron_id.as_str()) {
            return Err(format!("duplicate neuron id {}", neuron.neuron_id));
        }
    }
    Ok(())
}

/// Validate canonical fields derived for one SNS neuron row.
pub(in crate::sns::report) fn validate_sns_neuron_row(neuron: &SnsNeuronRow) -> Result<(), String> {
    if neuron.neuron_id.len() != SNS_NEURON_ID_HEX_LENGTH
        || !is_canonical_lowercase_hex(&neuron.neuron_id)
    {
        return Err(format!(
            "neuron id {} is not 32-byte canonical lowercase hexadecimal",
            neuron.neuron_id
        ));
    }

    let expected_created_at = format_utc_timestamp_secs(neuron.created_timestamp_seconds);
    if neuron.created_at != expected_created_at {
        return Err(format!(
            "neuron {} created_at does not match created_timestamp_seconds",
            neuron.neuron_id
        ));
    }
    Ok(())
}

/// Parse one exact 32-byte SNS neuron id from canonical lowercase hexadecimal text.
pub(in crate::sns::report) fn sns_neuron_id_from_text(
    neuron_id: &str,
) -> Result<SnsNeuronId, SnsHostError> {
    if neuron_id.len() != SNS_NEURON_ID_HEX_LENGTH || !is_lowercase_hex(neuron_id) {
        return Err(SnsHostError::InvalidNeuronIdText {
            neuron_id: neuron_id.to_string(),
        });
    }
    let id = (0..neuron_id.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&neuron_id[index..index + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| SnsHostError::InvalidNeuronIdText {
            neuron_id: neuron_id.to_string(),
        })?;
    Ok(SnsNeuronId { id })
}

/// Validate one exact neuron detail result returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_neuron(
    neuron: &MainnetSnsNeuron,
    requested_neuron_id: &str,
) -> Result<(), SnsHostError> {
    sns_neuron_id_from_text(requested_neuron_id)?;
    validate_sns_neuron_row(&neuron.detail.neuron).map_err(invalid_neuron_detail)?;
    if neuron.detail.neuron.neuron_id != requested_neuron_id {
        return Err(invalid_neuron_detail(format!(
            "returned neuron id {}, expected {requested_neuron_id}",
            neuron.detail.neuron.neuron_id
        )));
    }

    validate_neuron_permissions(&neuron.detail.permissions, true).map_err(invalid_neuron_detail)?;
    validate_maturity_disbursements(&neuron.detail.disburse_maturity_in_progress)
        .map_err(invalid_neuron_detail)?;

    let mut function_ids = HashSet::new();
    for followees in &neuron.detail.followees {
        if !function_ids.insert(followees.function_id) {
            return Err(invalid_neuron_detail(format!(
                "duplicate legacy followee function id {}",
                followees.function_id
            )));
        }
        validate_followee_ids(&followees.followee_neuron_ids)?;
    }

    if let Some(topic_followees) = neuron.detail.topic_followees.as_ref() {
        let mut topic_codes = HashSet::new();
        for topic in topic_followees {
            if !topic_codes.insert(topic.topic_code) {
                return Err(invalid_neuron_detail(format!(
                    "duplicate topic-following code {}",
                    topic.topic_code
                )));
            }
            if topic.topic.as_deref().is_some_and(str::is_empty) {
                return Err(invalid_neuron_detail(
                    "topic-following label must not be empty when present",
                ));
            }
            for followee in &topic.followees {
                if let Some(neuron_id) = followee.neuron_id.as_deref() {
                    validate_exact_neuron_id(neuron_id, "topic followee neuron id")?;
                }
            }
        }
    }

    let (mint, staking) = neuron.detail.derived_policy_observations();
    if neuron.detail.maturity_mint_conversion_observed_disabled != mint {
        return Err(invalid_neuron_detail(
            "maturity_mint_conversion_observed_disabled does not match raw evidence",
        ));
    }
    if neuron.detail.manual_maturity_staking_observed_disabled != staking {
        return Err(invalid_neuron_detail(
            "manual_maturity_staking_observed_disabled does not match raw evidence",
        ));
    }
    Ok(())
}

pub(super) fn validate_neuron_permissions(
    permissions: &[crate::sns::report::SnsNeuronPermissionRow],
    require_principal: bool,
) -> Result<(), String> {
    let mut principals = HashSet::new();
    for permission in permissions {
        let principal = match permission.principal.as_deref() {
            Some(principal) => {
                validate_canonical_principal_reason(principal, "permission principal")?;
                if !principals.insert(principal) {
                    return Err(format!("duplicate permission principal {principal}"));
                }
                principal
            }
            None if require_principal => {
                return Err("permission principal is missing".to_string());
            }
            None => "missing principal",
        };
        let mut codes = HashSet::new();
        for value in &permission.permission_types {
            if value.name != sns_neuron_permission_name(value.code) {
                return Err(format!(
                    "permission code {} has label {}, expected {}",
                    value.code,
                    value.name,
                    sns_neuron_permission_name(value.code)
                ));
            }
            if !codes.insert(value.code) {
                return Err(format!(
                    "{principal} contains duplicate permission code {}",
                    value.code
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_maturity_disbursements(
    disbursements: &[crate::sns::report::SnsMaturityDisbursementRow],
) -> Result<(), String> {
    for disbursement in disbursements {
        if let Some(account) = disbursement.account_to_disburse_to.as_ref() {
            if let Some(owner) = account.owner.as_deref() {
                validate_canonical_principal_reason(owner, "pending disbursement account owner")?;
            }
            if let Some(subaccount) = account.subaccount_hex.as_deref()
                && (subaccount.len() != SNS_NEURON_ID_HEX_LENGTH || !is_lowercase_hex(subaccount))
            {
                return Err(
                    "pending disbursement subaccount is not 32-byte lowercase hexadecimal text"
                        .to_string(),
                );
            }
        }
    }
    Ok(())
}

fn validate_followee_ids(neuron_ids: &[String]) -> Result<(), SnsHostError> {
    let mut unique = HashSet::new();
    for neuron_id in neuron_ids {
        validate_exact_neuron_id(neuron_id, "legacy followee neuron id")?;
        if !unique.insert(neuron_id) {
            return Err(invalid_neuron_detail(format!(
                "duplicate legacy followee neuron id {neuron_id}"
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_exact_neuron_id(neuron_id: &str, field: &str) -> Result<(), SnsHostError> {
    if neuron_id.len() == SNS_NEURON_ID_HEX_LENGTH && is_lowercase_hex(neuron_id) {
        Ok(())
    } else {
        Err(invalid_neuron_detail(format!(
            "{field} {neuron_id} is not 32-byte lowercase hexadecimal text"
        )))
    }
}

fn validate_canonical_principal_reason(value: &str, field: &str) -> Result<(), String> {
    let principal =
        Principal::from_text(value).map_err(|err| format!("{field} {value} is invalid: {err}"))?;
    if principal.to_text() == value {
        Ok(())
    } else {
        Err(format!("{field} {value} is not canonical principal text"))
    }
}

fn invalid_neuron_detail(reason: impl Into<String>) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: NEURON_DETAIL_CAPABILITY,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neuron_rows_require_canonical_ids_and_derived_created_at() {
        let valid = neuron(&"01".repeat(32));
        assert!(validate_sns_neuron_rows(std::slice::from_ref(&valid)).is_ok());

        let mut uppercase = valid.clone();
        uppercase.neuron_id = "0A".repeat(32);
        assert!(
            validate_sns_neuron_rows(&[uppercase])
                .expect_err("uppercase id rejected")
                .contains("canonical lowercase hexadecimal")
        );

        let mut mismatched_timestamp = valid;
        mismatched_timestamp.created_at = "not-derived".to_string();
        assert!(
            validate_sns_neuron_rows(&[mismatched_timestamp])
                .expect_err("mismatched timestamp rejected")
                .contains("created_at does not match")
        );
    }

    #[test]
    fn bounded_neuron_results_require_unique_ids_and_requested_limit() {
        let duplicate = MainnetSnsNeurons {
            neurons: vec![neuron(&"01".repeat(32)), neuron(&"01".repeat(32))],
        };
        assert!(matches!(
            validate_mainnet_sns_neurons(&duplicate, 2),
            Err(SnsHostError::InvalidSourceData {
                capability: "SNS neurons",
                ..
            })
        ));

        let over_limit = MainnetSnsNeurons {
            neurons: vec![neuron(&"01".repeat(32)), neuron(&"02".repeat(32))],
        };
        let error = validate_mainnet_sns_neurons(&over_limit, 1).expect_err("limit enforced");
        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS neurons",
                reason,
            } if reason == "returned 2 rows for requested limit 1"
        ));

        let page = MainnetSnsNeuronPage {
            neurons: over_limit.neurons,
            last_cursor: None,
        };
        assert!(matches!(
            validate_mainnet_sns_neuron_page(&page, 1),
            Err(SnsHostError::InvalidSourceData {
                capability: "SNS neuron page",
                ..
            })
        ));
    }

    fn neuron(neuron_id: &str) -> SnsNeuronRow {
        SnsNeuronRow {
            neuron_id: neuron_id.to_string(),
            cached_neuron_stake_e8s: 100,
            maturity_e8s_equivalent: 10,
            staked_maturity_e8s_equivalent: None,
            created_timestamp_seconds: 1_700_000_000,
            created_at: "2023-11-14T22:13:20Z".to_string(),
            source_nns_neuron_id: None,
            auto_stake_maturity: None,
            aging_since_timestamp_seconds: 1_700_000_000,
            dissolve_state: None,
            voting_power_percentage_multiplier: 100,
            vesting_period_seconds: None,
            neuron_fees_e8s: 0,
        }
    }
}
