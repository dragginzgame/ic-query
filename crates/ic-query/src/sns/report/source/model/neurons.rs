//! Module: sns::report::source::model::neurons
//!
//! Responsibility: source-layer SNS neuron models.
//! Does not own: governance transport, cache storage, sorting, or rendering.
//! Boundary: carries live neuron rows and pagination cursors into builders.

use crate::{
    hex::is_canonical_lowercase_hex,
    sns::report::{SnsHostError, SnsNeuronRow},
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::{CandidType, Deserialize};
use std::collections::HashSet;

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
    if neurons.len() > requested_limit as usize {
        return Err(SnsHostError::InvalidSourceData {
            capability,
            reason: format!(
                "returned {} rows for requested limit {requested_limit}",
                neurons.len()
            ),
        });
    }
    validate_sns_neuron_rows(neurons)
        .map_err(|reason| SnsHostError::InvalidSourceData { capability, reason })
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
    if !is_canonical_lowercase_hex(&neuron.neuron_id) {
        return Err(format!(
            "neuron id {} is not canonical lowercase hexadecimal",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neuron_rows_require_canonical_ids_and_derived_created_at() {
        let valid = neuron("01");
        assert!(validate_sns_neuron_rows(std::slice::from_ref(&valid)).is_ok());

        let mut uppercase = valid.clone();
        uppercase.neuron_id = "0A".to_string();
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
            neurons: vec![neuron("01"), neuron("01")],
        };
        assert!(matches!(
            validate_mainnet_sns_neurons(&duplicate, 2),
            Err(SnsHostError::InvalidSourceData {
                capability: "SNS neurons",
                ..
            })
        ));

        let over_limit = MainnetSnsNeurons {
            neurons: vec![neuron("01"), neuron("02")],
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
