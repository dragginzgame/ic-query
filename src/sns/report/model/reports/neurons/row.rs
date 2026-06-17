//! Module: sns::report::model::reports::neurons::row
//!
//! Responsibility: SNS neuron row DTO.
//! Does not own: governance wire conversion, cache sorting, or rendering.
//! Boundary: preserves raw neuron fields used by live reports and snapshots.

use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// SnsNeuronRow
///
/// Serializable row for one SNS neuron in live reports and cached snapshots.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsNeuronRow {
    pub neuron_id: String,
    pub cached_neuron_stake_e8s: u64,
    pub maturity_e8s_equivalent: u64,
    pub staked_maturity_e8s_equivalent: Option<u64>,
    pub created_timestamp_seconds: u64,
    pub created_at: String,
}
