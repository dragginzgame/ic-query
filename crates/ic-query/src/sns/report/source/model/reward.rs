//! Module: sns::report::source::model::reward
//!
//! Responsibility: source-layer SNS reward page models and strict collection invariants.
//! Does not own: live transport, checkpoint assembly, or text rendering.
//! Boundary: rejects non-canonical rows, pagination overlap, cursor drift, and post-exhaustion rows.

use super::neurons::{validate_maturity_disbursements, validate_neuron_permissions};
use crate::{
    hex::{hex_bytes, is_lowercase_hex},
    sns::report::{
        SNS_REWARD_CHECKPOINT_PAGE_SIZE, SnsHostError, SnsNeuronId, SnsRewardCheckpointRow,
    },
};

const CAPABILITY: &str = "SNS reward checkpoint";
const NEURON_ID_HEX_LENGTH: usize = 64;

///
/// MainnetSnsRewardNeuronPage
///
/// One strict variable-evidence neuron page returned by a reward source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsRewardNeuronPage {
    /// Native neuron rows in exclusive-cursor order.
    pub neurons: Vec<SnsRewardCheckpointRow>,
    /// Final row id advertised only when the page is exactly full.
    pub next_cursor: Option<SnsNeuronId>,
}

///
/// SnsRewardCollectionState
///
/// Strict ordered state for one API-exhausted reward-neuron walk.
///

pub(in crate::sns::report) struct SnsRewardCollectionState {
    rows: Vec<SnsRewardCheckpointRow>,
    page_count: u32,
    next_cursor: Option<SnsNeuronId>,
    exhausted: bool,
}

impl SnsRewardCollectionState {
    pub(in crate::sns::report) const fn new() -> Self {
        Self {
            rows: Vec::new(),
            page_count: 0,
            next_cursor: None,
            exhausted: false,
        }
    }

    pub(in crate::sns::report) const fn page_count(&self) -> u32 {
        self.page_count
    }

    pub(in crate::sns::report) const fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub(in crate::sns::report) const fn next_cursor(&self) -> Option<&SnsNeuronId> {
        self.next_cursor.as_ref()
    }

    pub(in crate::sns::report) const fn exhausted(&self) -> bool {
        self.exhausted
    }

    pub(in crate::sns::report) fn ingest_page(
        &mut self,
        page: MainnetSnsRewardNeuronPage,
    ) -> Result<(), SnsHostError> {
        if self.exhausted {
            return Err(invalid("source offered rows after reported API exhaustion"));
        }
        validate_mainnet_sns_reward_neuron_page(&page)?;
        if let (Some(previous), Some(first)) = (self.rows.last(), page.neurons.first())
            && first.neuron_id <= previous.neuron_id
        {
            return Err(invalid(format!(
                "neuron id {} does not increase after previous id {}",
                first.neuron_id, previous.neuron_id
            )));
        }
        self.page_count =
            self.page_count
                .checked_add(1)
                .ok_or(SnsHostError::RewardCheckpointArithmetic {
                    field: "page_count",
                })?;
        self.rows.extend(page.neurons);
        self.next_cursor = page.next_cursor;
        self.exhausted = self.next_cursor.is_none();
        Ok(())
    }

    pub(in crate::sns::report) fn into_rows(self) -> Vec<SnsRewardCheckpointRow> {
        self.rows
    }
}

pub(in crate::sns::report) fn validate_mainnet_sns_reward_neuron_page(
    page: &MainnetSnsRewardNeuronPage,
) -> Result<(), SnsHostError> {
    if page.neurons.len() > SNS_REWARD_CHECKPOINT_PAGE_SIZE as usize {
        return Err(invalid(format!(
            "returned {} rows for fixed page size {SNS_REWARD_CHECKPOINT_PAGE_SIZE}",
            page.neurons.len()
        )));
    }
    for row in &page.neurons {
        validate_reward_checkpoint_row(row)?;
    }
    for pair in page.neurons.windows(2) {
        if pair[0].neuron_id >= pair[1].neuron_id {
            return Err(invalid(format!(
                "neuron ids are not strictly increasing: {} then {}",
                pair[0].neuron_id, pair[1].neuron_id
            )));
        }
    }
    validate_page_cursor(page)
}

pub(in crate::sns::report) fn validate_reward_checkpoint_row(
    row: &SnsRewardCheckpointRow,
) -> Result<(), SnsHostError> {
    if row.neuron_id.len() != NEURON_ID_HEX_LENGTH || !is_lowercase_hex(&row.neuron_id) {
        return Err(invalid(format!(
            "neuron id {} is not 32-byte lowercase hexadecimal text",
            row.neuron_id
        )));
    }
    let combined =
        row.checked_combined_maturity()
            .ok_or(SnsHostError::RewardCheckpointArithmetic {
                field: "combined_maturity_e8s_equivalent",
            })?;
    if row.combined_maturity_e8s_equivalent != combined {
        return Err(invalid(format!(
            "neuron {} combined maturity does not match raw components",
            row.neuron_id
        )));
    }
    validate_neuron_permissions(&row.permissions, false).map_err(invalid)?;
    validate_maturity_disbursements(&row.disburse_maturity_in_progress).map_err(invalid)?;
    let (mint, staking) = row.derived_policy_observations();
    if row.maturity_mint_conversion_observed_disabled != mint
        || row.manual_maturity_staking_observed_disabled != staking
    {
        return Err(invalid(format!(
            "neuron {} policy observations do not match raw evidence",
            row.neuron_id
        )));
    }
    Ok(())
}

fn validate_page_cursor(page: &MainnetSnsRewardNeuronPage) -> Result<(), SnsHostError> {
    if page.neurons.len() == SNS_REWARD_CHECKPOINT_PAGE_SIZE as usize {
        let cursor = page
            .next_cursor
            .as_ref()
            .ok_or_else(|| invalid("full page does not advertise its final neuron id"))?;
        let cursor_text = hex_bytes(&cursor.id);
        let final_id = &page
            .neurons
            .last()
            .expect("full page has a final row")
            .neuron_id;
        if cursor_text != *final_id {
            return Err(invalid(format!(
                "full-page cursor {cursor_text} does not equal final neuron id {final_id}"
            )));
        }
    } else if page.next_cursor.is_some() {
        return Err(invalid(
            "short or empty exhaustion page must not advertise a cursor",
        ));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: CAPABILITY,
        reason: reason.into(),
    }
}
