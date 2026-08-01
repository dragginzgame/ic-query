//! Module: sns::report::model::reports::neurons::row
//!
//! Responsibility: SNS neuron row DTO.
//! Does not own: governance wire conversion, row ordering, or rendering.
//! Boundary: preserves raw neuron fields used by live reports and snapshots.

use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// SnsNeuronDissolveState
///
/// Raw dissolve-state alternative returned by SNS governance.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum SnsNeuronDissolveState {
    /// Remaining dissolve delay in seconds.
    DissolveDelaySeconds(u64),
    /// Unix timestamp in seconds at which the neuron is dissolved.
    WhenDissolvedTimestampSeconds(u64),
}

///
/// SnsNeuronRow
///
/// Serializable row for one SNS neuron in live reports and cached snapshots.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronRow {
    /// Lowercase hexadecimal SNS neuron identifier.
    pub neuron_id: String,
    /// Cached neuron stake in e8s.
    pub cached_neuron_stake_e8s: u64,
    /// Unstaked maturity in e8s-equivalent units.
    pub maturity_e8s_equivalent: u64,
    /// Staked maturity in e8s-equivalent units when present.
    pub staked_maturity_e8s_equivalent: Option<u64>,
    /// Unix timestamp in seconds at which the neuron was created.
    pub created_timestamp_seconds: u64,
    /// UTC rendering derived exactly from `created_timestamp_seconds`.
    pub created_at: String,
    /// Source NNS neuron identifier when the neuron was created from one.
    pub source_nns_neuron_id: Option<u64>,
    /// Whether maturity is automatically staked when governance supplies the setting.
    pub auto_stake_maturity: Option<bool>,
    /// Raw SNS governance aging timestamp in seconds.
    pub aging_since_timestamp_seconds: u64,
    /// Raw SNS governance dissolve-state alternative.
    pub dissolve_state: Option<SnsNeuronDissolveState>,
    /// Raw voting-power percentage multiplier.
    pub voting_power_percentage_multiplier: u64,
    /// Vesting period in seconds when present.
    pub vesting_period_seconds: Option<u64>,
    /// Accumulated neuron fees in e8s.
    pub neuron_fees_e8s: u64,
}
