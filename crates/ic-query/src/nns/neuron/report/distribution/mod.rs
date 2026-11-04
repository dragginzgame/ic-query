//! Module: nns::neuron::report::distribution
//!
//! Responsibility: validate and aggregate complete caller-retained public NNS neuron collections.
//! Does not own: collection transport, persistence, cache policy, or process output.
//! Boundary: projects complete public `NeuronInfo` evidence into portable distributions.

use super::{
    NnsNeuronCollectionState, collection::validate_collection_state, model::NnsNeuronRow,
    source::validate_neuron_rows,
};
use crate::nns::governance::{NnsGovernanceCollectionStatus, NnsGovernanceSourceProvenance};
use serde::Serialize;
use std::collections::BTreeMap;
use thiserror::Error as ThisError;

/// Version of the portable NNS public-neuron distribution report schema.
pub const NNS_NEURON_DISTRIBUTION_REPORT_SCHEMA_VERSION: u32 = 1;

///
/// NnsNeuronStateDistribution
///
/// Public neuron count and effective stake for one raw Governance state code.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronStateDistribution {
    /// Raw native Governance state code.
    pub state: i32,
    /// Number of collected neurons with this state code.
    pub neuron_count: u64,
    /// Sum of public effective stake for neurons with this state code.
    pub effective_stake_e8s: u64,
}

///
/// NnsNeuronVisibilityDistribution
///
/// Public neuron count and effective stake for one optional Governance visibility code.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronVisibilityDistribution {
    /// Raw optional native Governance visibility code.
    pub visibility: Option<i32>,
    /// Number of collected neurons with this visibility code or absence.
    pub neuron_count: u64,
    /// Sum of public effective stake for neurons with this visibility code or absence.
    pub effective_stake_e8s: u64,
}

///
/// NnsNeuronTypeDistribution
///
/// Public neuron count and effective stake for one optional Governance neuron-type code.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronTypeDistribution {
    /// Raw optional native Governance neuron-type code.
    pub neuron_type: Option<i32>,
    /// Number of collected neurons with this type code or absence.
    pub neuron_count: u64,
    /// Sum of public effective stake for neurons with this type code or absence.
    pub effective_stake_e8s: u64,
}

///
/// NnsNeuronDistributionReport
///
/// Deterministic local distribution over one complete public NNS neuron collection.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronDistributionReport {
    /// Shared record, flattened in JSON.
    #[serde(flatten)]
    pub context: crate::nns::governance::NnsGovernanceAnalyticsContext,
    /// Number of admitted public neuron rows in the complete collection.
    pub collected_neuron_count: u64,
    /// Earliest raw Governance retrieval timestamp among collected rows.
    pub earliest_retrieved_at_timestamp_seconds: Option<u64>,
    /// Latest raw Governance retrieval timestamp among collected rows.
    pub latest_retrieved_at_timestamp_seconds: Option<u64>,
    /// Sum of public effective stake across every collected neuron.
    pub total_effective_stake_e8s: u64,
    /// Rows carrying a public staked-maturity value.
    pub reported_staked_maturity_neuron_count: u64,
    /// Rows without a public staked-maturity value.
    pub unreported_staked_maturity_neuron_count: u64,
    /// Sum of reported staked maturity values only.
    pub total_reported_staked_maturity_e8s_equivalent: u64,
    /// Rows carrying current deciding voting power.
    pub reported_deciding_voting_power_neuron_count: u64,
    /// Rows without current deciding voting power.
    pub unreported_deciding_voting_power_neuron_count: u64,
    /// Sum of reported current deciding voting power values only.
    pub total_reported_deciding_voting_power: u64,
    /// Rows carrying current potential voting power.
    pub reported_potential_voting_power_neuron_count: u64,
    /// Rows without current potential voting power.
    pub unreported_potential_voting_power_neuron_count: u64,
    /// Sum of reported current potential voting power values only.
    pub total_reported_potential_voting_power: u64,
    /// Rows carrying registered known-neuron metadata.
    pub known_neuron_metadata_count: u64,
    /// Rows carrying a public Neurons' Fund join timestamp.
    pub neurons_fund_join_timestamp_present_count: u64,
    /// Canonically raw-code-ordered distribution by neuron state.
    pub state_distribution: Vec<NnsNeuronStateDistribution>,
    /// Canonically optional-raw-code-ordered distribution by visibility.
    pub visibility_distribution: Vec<NnsNeuronVisibilityDistribution>,
    /// Canonically optional-raw-code-ordered distribution by neuron type.
    pub neuron_type_distribution: Vec<NnsNeuronTypeDistribution>,
}

///
/// NnsNeuronDistributionError
///
/// Deterministic validation or accounting failure from local neuron distribution projection.
///

#[derive(Debug, ThisError)]
pub enum NnsNeuronDistributionError {
    /// The supplied collection state failed its shared continuation invariants.
    #[error("invalid NNS neuron collection state for distribution projection: {reason}")]
    InvalidCollectionState {
        /// Deterministic collection invariant failure.
        reason: String,
    },

    /// The collection stopped without observing Governance API exhaustion.
    #[error("NNS neuron distribution requires a complete collection; state is {status}")]
    CollectionNotComplete {
        /// Current lifecycle of the otherwise valid collection state.
        status: NnsGovernanceCollectionStatus,
    },

    /// The supplied rows do not match the collection's admitted-row accounting.
    #[error(
        "NNS neuron distribution received {actual} rows; complete collection accounts for {expected}"
    )]
    NeuronCountMismatch {
        /// Neuron rows accounted for by the collection state.
        expected: u64,
        /// Neuron rows supplied to the builder.
        actual: u64,
    },

    /// Supplied rows failed the shared public-neuron response contract.
    #[error("invalid NNS neuron rows for distribution projection: {reason}")]
    InvalidNeuronRows {
        /// Deterministic row invariant failure.
        reason: String,
    },

    /// A count conversion, increment, or numeric sum exceeded `u64`.
    #[error("NNS neuron distribution accounting overflow while updating {field}")]
    AccountingOverflow {
        /// Count or sum that exceeded its representation.
        field: &'static str,
    },
}

/// Build one deterministic distribution from a complete caller-retained public-neuron collection.
pub fn build_nns_neuron_distribution_report(
    collection: &NnsNeuronCollectionState,
    neurons: &[NnsNeuronRow],
) -> Result<NnsNeuronDistributionReport, NnsNeuronDistributionError> {
    validate_collection_state(collection).map_err(|error| {
        NnsNeuronDistributionError::InvalidCollectionState {
            reason: error.to_string(),
        }
    })?;
    if !collection.is_complete() {
        return Err(NnsNeuronDistributionError::CollectionNotComplete {
            status: collection.status(),
        });
    }
    let source = collection.source().cloned().ok_or_else(|| {
        NnsNeuronDistributionError::InvalidCollectionState {
            reason: "complete collection has no concrete source provenance".to_string(),
        }
    })?;

    let expected = collection.neurons_fetched();
    let actual = u64::try_from(neurons.len()).map_err(|_| {
        NnsNeuronDistributionError::AccountingOverflow {
            field: "supplied_neuron_count",
        }
    })?;
    if actual != expected {
        return Err(NnsNeuronDistributionError::NeuronCountMismatch { expected, actual });
    }
    validate_neuron_rows(neurons).map_err(|error| {
        NnsNeuronDistributionError::InvalidNeuronRows {
            reason: error.to_string(),
        }
    })?;

    aggregate_distribution(collection, neurons, source)
}

#[cfg(feature = "nns-host")]
pub(super) fn build_distribution_from_cache(
    cache: &super::cache::AdmittedNeuronCache,
) -> Result<NnsNeuronDistributionReport, NnsNeuronDistributionError> {
    let collection = cache.collection_state();
    let source = collection.source().cloned().ok_or_else(|| {
        NnsNeuronDistributionError::InvalidCollectionState {
            reason: "complete collection has no concrete source provenance".to_string(),
        }
    })?;
    aggregate_distribution(collection, cache.rows(), source)
}

fn aggregate_distribution(
    collection: &NnsNeuronCollectionState,
    neurons: &[NnsNeuronRow],
    source: NnsGovernanceSourceProvenance,
) -> Result<NnsNeuronDistributionReport, NnsNeuronDistributionError> {
    let mut distribution = DistributionAccumulator::default();
    for neuron in neurons {
        distribution.observe(neuron)?;
    }
    Ok(distribution.into_report(collection, source))
}

#[derive(Default)]
struct DistributionTotals {
    count: u64,
    total: u64,
}

#[derive(Default)]
struct DistributionAccumulator {
    states: BTreeMap<i32, DistributionTotals>,
    visibilities: BTreeMap<Option<i32>, DistributionTotals>,
    neuron_types: BTreeMap<Option<i32>, DistributionTotals>,
    total_effective_stake: DistributionTotals,
    staked_maturity: DistributionTotals,
    deciding_voting_power: DistributionTotals,
    potential_voting_power: DistributionTotals,
    known_neuron_metadata_count: u64,
    neurons_fund_join_timestamp_present_count: u64,
    retrieved_timestamp_bounds: Option<(u64, u64)>,
}

impl DistributionAccumulator {
    fn observe(&mut self, neuron: &NnsNeuronRow) -> Result<(), NnsNeuronDistributionError> {
        record_distribution(
            &mut self.total_effective_stake,
            Some(neuron.stake_e8s),
            "collected_neuron_count",
            "total_effective_stake_e8s",
        )?;
        record_distribution(
            self.states.entry(neuron.state).or_default(),
            Some(neuron.stake_e8s),
            "state_neuron_count",
            "state_effective_stake_e8s",
        )?;
        record_distribution(
            self.visibilities.entry(neuron.visibility).or_default(),
            Some(neuron.stake_e8s),
            "visibility_neuron_count",
            "visibility_effective_stake_e8s",
        )?;
        record_distribution(
            self.neuron_types.entry(neuron.neuron_type).or_default(),
            Some(neuron.stake_e8s),
            "neuron_type_neuron_count",
            "neuron_type_effective_stake_e8s",
        )?;
        record_distribution(
            &mut self.staked_maturity,
            neuron.staked_maturity_e8s_equivalent,
            "reported_staked_maturity_neuron_count",
            "total_reported_staked_maturity_e8s_equivalent",
        )?;
        record_distribution(
            &mut self.deciding_voting_power,
            neuron.deciding_voting_power,
            "reported_deciding_voting_power_neuron_count",
            "total_reported_deciding_voting_power",
        )?;
        record_distribution(
            &mut self.potential_voting_power,
            neuron.potential_voting_power,
            "reported_potential_voting_power_neuron_count",
            "total_reported_potential_voting_power",
        )?;
        if neuron.known_neuron_data.is_some() {
            increment(
                &mut self.known_neuron_metadata_count,
                "known_neuron_metadata_count",
            )?;
        }
        if neuron.joined_community_fund_timestamp_seconds.is_some() {
            increment(
                &mut self.neurons_fund_join_timestamp_present_count,
                "neurons_fund_join_timestamp_present_count",
            )?;
        }
        let retrieved_at = neuron.retrieved_at_timestamp_seconds;
        self.retrieved_timestamp_bounds = Some(match self.retrieved_timestamp_bounds {
            Some((earliest, latest)) => (earliest.min(retrieved_at), latest.max(retrieved_at)),
            None => (retrieved_at, retrieved_at),
        });
        Ok(())
    }

    fn into_report(
        self,
        collection: &NnsNeuronCollectionState,
        source: NnsGovernanceSourceProvenance,
    ) -> NnsNeuronDistributionReport {
        let collected_neuron_count = collection.neurons_fetched();
        NnsNeuronDistributionReport {
            context: crate::nns::governance::NnsGovernanceAnalyticsContext {
                schema_version: NNS_NEURON_DISTRIBUTION_REPORT_SCHEMA_VERSION,
                network: collection.network().to_string(),
                governance_canister_id: collection.governance_canister_id().to_string(),
                source,
                collection_started_at: collection.started_at().to_string(),
                collection_updated_at: collection.updated_at().to_string(),
                collection_page_count: collection.pages_fetched(),
                point_in_time_guaranteed: false,
            },
            collected_neuron_count,
            earliest_retrieved_at_timestamp_seconds: self
                .retrieved_timestamp_bounds
                .map(|bounds| bounds.0),
            latest_retrieved_at_timestamp_seconds: self
                .retrieved_timestamp_bounds
                .map(|bounds| bounds.1),
            total_effective_stake_e8s: self.total_effective_stake.total,
            reported_staked_maturity_neuron_count: self.staked_maturity.count,
            unreported_staked_maturity_neuron_count: collected_neuron_count
                - self.staked_maturity.count,
            total_reported_staked_maturity_e8s_equivalent: self.staked_maturity.total,
            reported_deciding_voting_power_neuron_count: self.deciding_voting_power.count,
            unreported_deciding_voting_power_neuron_count: collected_neuron_count
                - self.deciding_voting_power.count,
            total_reported_deciding_voting_power: self.deciding_voting_power.total,
            reported_potential_voting_power_neuron_count: self.potential_voting_power.count,
            unreported_potential_voting_power_neuron_count: collected_neuron_count
                - self.potential_voting_power.count,
            total_reported_potential_voting_power: self.potential_voting_power.total,
            known_neuron_metadata_count: self.known_neuron_metadata_count,
            neurons_fund_join_timestamp_present_count: self
                .neurons_fund_join_timestamp_present_count,
            state_distribution: self
                .states
                .into_iter()
                .map(|(state, distribution)| NnsNeuronStateDistribution {
                    state,

                    neuron_count: distribution.count,
                    effective_stake_e8s: distribution.total,
                })
                .collect(),
            visibility_distribution: self
                .visibilities
                .into_iter()
                .map(
                    |(visibility, distribution)| NnsNeuronVisibilityDistribution {
                        visibility,

                        neuron_count: distribution.count,
                        effective_stake_e8s: distribution.total,
                    },
                )
                .collect(),
            neuron_type_distribution: self
                .neuron_types
                .into_iter()
                .map(|(neuron_type, distribution)| NnsNeuronTypeDistribution {
                    neuron_type,

                    neuron_count: distribution.count,
                    effective_stake_e8s: distribution.total,
                })
                .collect(),
        }
    }
}

fn increment(value: &mut u64, field: &'static str) -> Result<(), NnsNeuronDistributionError> {
    *value = value
        .checked_add(1)
        .ok_or(NnsNeuronDistributionError::AccountingOverflow { field })?;
    Ok(())
}

fn record_distribution(
    values: &mut DistributionTotals,
    value: Option<u64>,
    count_field: &'static str,
    total_field: &'static str,
) -> Result<(), NnsNeuronDistributionError> {
    if let Some(value) = value {
        let count = values
            .count
            .checked_add(1)
            .ok_or(NnsNeuronDistributionError::AccountingOverflow { field: count_field })?;
        let total = values
            .total
            .checked_add(value)
            .ok_or(NnsNeuronDistributionError::AccountingOverflow { field: total_field })?;
        *values = DistributionTotals { count, total };
    }
    Ok(())
}

#[cfg(test)]
mod tests;
