//! Module: sns::report::source::model::neurons
//!
//! Responsibility: source-layer SNS neuron models.
//! Does not own: governance transport, cache storage, sorting, or rendering.
//! Boundary: carries live neuron rows and pagination cursors into builders.

use super::validation::SnsSourceValidator;
use crate::{
    hex::{decode_lowercase_hex, is_lowercase_hex_32_bytes},
    sns::report::{
        SnsHostError, SnsNeuronDetail, SnsNeuronRow,
        model::reports::neurons::detail::{
            validate_maturity_disbursements, validate_neuron_permissions,
        },
    },
};
use ic_platform::candid::contract::{CandidType, serde::Deserialize};
use std::collections::HashSet;

const SNS_NEURON_ID_HEX_LENGTH: usize = 64;
const NEURON_DETAIL_CAPABILITY: &str = "SNS neuron detail";

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
#[candid_path("ic_platform::candid::contract")]
pub struct SnsNeuronId {
    pub id: Vec<u8>,
}

/// Validate one bounded neuron result returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_neurons(
    neurons: &[SnsNeuronRow],
    requested_limit: u32,
) -> Result<(), SnsHostError> {
    let validator = SnsSourceValidator::new("SNS neurons");
    validator.row_limit(neurons.len(), requested_limit)?;
    validate_sns_neuron_rows(neurons).map_err(|reason| validator.invalid(reason))
}

/// Validate one neuron page returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_neuron_page(
    page: &MainnetSnsNeuronPage,
    requested_limit: u32,
    start_page_at: Option<&SnsNeuronId>,
) -> Result<(), SnsHostError> {
    let validator = SnsSourceValidator::new("SNS neuron page");
    validator.row_limit(page.neurons.len(), requested_limit)?;
    for neuron in &page.neurons {
        validate_sns_neuron_row(neuron).map_err(|reason| validator.invalid(reason))?;
    }
    let last_id = page.neurons.last().map(|neuron| neuron.neuron_id.as_str());
    let cursor_text = page
        .last_cursor
        .as_ref()
        .map(|cursor| crate::hex::hex_bytes(&cursor.id));
    if (cursor_text.is_some() || page.neurons.len() == requested_limit as usize)
        && last_id != cursor_text.as_deref()
    {
        return Err(validator.invalid("cursor does not match the final neuron id".to_string()));
    }
    if page
        .neurons
        .windows(2)
        .any(|rows| rows[0].neuron_id >= rows[1].neuron_id)
    {
        return Err(validator.invalid("neuron page is not in ascending id order".to_string()));
    }
    if let Some(start) = start_page_at {
        let start_text = crate::hex::hex_bytes(&start.id);
        // Governance may include the boundary neuron again; older rows are never valid.
        if page
            .neurons
            .first()
            .is_some_and(|row| row.neuron_id < start_text)
        {
            return Err(validator.invalid("neuron page precedes its requested cursor".to_string()));
        }
        if page.neurons.len() == requested_limit as usize
            && cursor_text
                .as_ref()
                .is_none_or(|cursor| cursor <= &start_text)
        {
            return Err(
                validator.invalid("full neuron page does not advance its cursor".to_string())
            );
        }
    }
    Ok(())
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
    if !is_lowercase_hex_32_bytes(&neuron.neuron_id) {
        return Err(format!(
            "neuron id {} is not 32-byte canonical lowercase hexadecimal",
            neuron.neuron_id
        ));
    }

    Ok(())
}

/// Parse one exact 32-byte SNS neuron id from canonical lowercase hexadecimal text.
pub(in crate::sns::report) fn sns_neuron_id_from_text(
    neuron_id: &str,
) -> Result<SnsNeuronId, SnsHostError> {
    let id = if neuron_id.len() == SNS_NEURON_ID_HEX_LENGTH {
        decode_lowercase_hex(neuron_id)
    } else {
        None
    }
    .ok_or_else(|| SnsHostError::InvalidNeuronIdText {
        neuron_id: neuron_id.to_string(),
    })?;
    Ok(SnsNeuronId { id })
}

/// Validate one exact neuron detail result returned by a public source implementation.
pub(in crate::sns::report) fn validate_mainnet_sns_neuron(
    neuron: &SnsNeuronDetail,
    requested_neuron_id: &str,
) -> Result<(), SnsHostError> {
    sns_neuron_id_from_text(requested_neuron_id)?;
    validate_sns_neuron_row(&neuron.neuron).map_err(invalid_neuron_detail)?;
    if neuron.neuron.neuron_id != requested_neuron_id {
        return Err(invalid_neuron_detail(format!(
            "returned neuron id {}, expected {requested_neuron_id}",
            neuron.neuron.neuron_id
        )));
    }

    validate_neuron_permissions(&neuron.permissions).map_err(invalid_neuron_detail)?;
    validate_maturity_disbursements(&neuron.disburse_maturity_in_progress)
        .map_err(invalid_neuron_detail)?;

    let mut function_ids = HashSet::new();
    for followees in &neuron.followees {
        if !function_ids.insert(followees.function_id) {
            return Err(invalid_neuron_detail(format!(
                "duplicate legacy followee function id {}",
                followees.function_id
            )));
        }
        validate_followee_ids(&followees.followee_neuron_ids)?;
    }

    if let Some(topic_followees) = neuron.topic_followees.as_ref() {
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
    if is_lowercase_hex_32_bytes(neuron_id) {
        Ok(())
    } else {
        Err(invalid_neuron_detail(format!(
            "{field} {neuron_id} is not 32-byte lowercase hexadecimal text"
        )))
    }
}

fn invalid_neuron_detail(reason: impl Into<String>) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: NEURON_DETAIL_CAPABILITY,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
