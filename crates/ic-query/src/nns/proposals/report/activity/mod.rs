//! Module: nns::proposals::report::activity
//!
//! Responsibility: validate and aggregate complete caller-retained NNS proposal collections.
//! Does not own: collection transport, persistence, cache policy, or process output.
//! Boundary: projects complete proposal evidence into deterministic portable activity reports.

use super::{
    NnsProposalCollectionState, collection::validate_collection_state, model::NnsProposalRow,
};
use crate::nns::governance::{NnsGovernanceCollectionStatus, NnsGovernanceSourceProvenance};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use thiserror::Error as ThisError;

/// Version of the portable NNS proposal activity report schema.
pub const NNS_PROPOSAL_ACTIVITY_REPORT_SCHEMA_VERSION: u32 = 1;

const SECONDS_PER_DAY: u64 = 86_400;

///
/// NnsProposalActivityRequest
///
/// Optional half-open proposal-creation time window for one local activity projection.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NnsProposalActivityRequest {
    /// Inclusive lower proposal-creation timestamp bound.
    pub from_proposal_timestamp_seconds: Option<u64>,
    /// Exclusive upper proposal-creation timestamp bound.
    pub until_proposal_timestamp_seconds: Option<u64>,
}

///
/// NnsProposalTopicCount
///
/// Proposal count for one raw native Governance topic code.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalTopicCount {
    /// Raw native Governance topic code.
    pub topic: i32,
    /// Number of included proposals with this topic code.
    pub proposal_count: u64,
}

///
/// NnsProposalStatusCount
///
/// Proposal count for one raw native Governance decision-status code.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalStatusCount {
    /// Raw native Governance decision-status code.
    pub status: i32,
    /// Number of included proposals with this status code.
    pub proposal_count: u64,
}

///
/// NnsProposalRewardStatusCount
///
/// Proposal count for one raw native Governance reward-status code.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalRewardStatusCount {
    /// Raw native Governance reward-status code.
    pub reward_status: i32,
    /// Number of included proposals with this reward-status code.
    pub proposal_count: u64,
}

///
/// NnsProposalDayCount
///
/// Proposal count for one UTC proposal-creation day.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalDayCount {
    /// Unix timestamp at 00:00:00 UTC for the represented day.
    pub day_start_timestamp_seconds: u64,
    /// Number of included proposals created during this UTC day.
    pub proposal_count: u64,
}

///
/// NnsProposalActivityReport
///
/// Deterministic local activity projection over one complete NNS proposal collection.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalActivityReport {
    /// Shared record, flattened in JSON.
    #[serde(flatten)]
    pub context: crate::nns::governance::NnsGovernanceAnalyticsContext,
    /// Number of admitted proposal rows in the complete collection.
    pub collected_proposal_count: u64,
    /// Inclusive lower proposal-creation timestamp bound requested by the caller.
    pub from_proposal_timestamp_seconds: Option<u64>,
    /// Exclusive upper proposal-creation timestamp bound requested by the caller.
    pub until_proposal_timestamp_seconds: Option<u64>,
    /// Number of proposals included by the local time window.
    pub included_proposal_count: u64,
    /// Number of proposals excluded before the inclusive lower bound.
    pub excluded_before_from_count: u64,
    /// Number of proposals excluded at or after the exclusive upper bound.
    pub excluded_at_or_after_until_count: u64,
    /// Earliest creation timestamp among included proposals.
    pub earliest_included_proposal_timestamp_seconds: Option<u64>,
    /// Latest creation timestamp among included proposals.
    pub latest_included_proposal_timestamp_seconds: Option<u64>,
    /// Canonically raw-code-ordered proposal counts by topic.
    pub topic_counts: Vec<NnsProposalTopicCount>,
    /// Canonically raw-code-ordered proposal counts by decision status.
    pub status_counts: Vec<NnsProposalStatusCount>,
    /// Canonically raw-code-ordered proposal counts by reward status.
    pub reward_status_counts: Vec<NnsProposalRewardStatusCount>,
    /// Canonically time-ordered proposal counts by UTC creation day.
    pub day_counts: Vec<NnsProposalDayCount>,
}

///
/// NnsProposalActivityError
///
/// Deterministic validation or accounting failure from local proposal activity projection.
///

#[derive(Debug, ThisError)]
pub enum NnsProposalActivityError {
    /// The supplied collection state failed its shared continuation invariants.
    #[error("invalid NNS proposal collection state for activity projection: {reason}")]
    InvalidCollectionState {
        /// Deterministic collection invariant failure.
        reason: String,
    },

    /// The collection stopped without observing Governance API exhaustion.
    #[error("NNS proposal activity requires a complete collection; state is {status}")]
    CollectionNotComplete {
        /// Current lifecycle of the otherwise valid collection state.
        status: NnsGovernanceCollectionStatus,
    },

    /// The requested half-open proposal time window is empty or reversed.
    #[error(
        "invalid NNS proposal activity time window: from {from_proposal_timestamp_seconds} must be below until {until_proposal_timestamp_seconds}"
    )]
    InvalidTimeWindow {
        /// Inclusive lower proposal-creation timestamp bound.
        from_proposal_timestamp_seconds: u64,
        /// Exclusive upper proposal-creation timestamp bound.
        until_proposal_timestamp_seconds: u64,
    },

    /// The supplied rows do not match the collection's admitted-row accounting.
    #[error(
        "NNS proposal activity received {actual} rows; complete collection accounts for {expected}"
    )]
    ProposalCountMismatch {
        /// Proposal rows accounted for by the collection state.
        expected: u64,
        /// Proposal rows supplied to the builder.
        actual: u64,
    },

    /// A supplied row has no proposal identifier.
    #[error("NNS proposal activity received a row without a proposal id")]
    MissingProposalId,

    /// A supplied row uses the reserved zero proposal identifier.
    #[error("NNS proposal activity received proposal id zero")]
    ZeroProposalId,

    /// A supplied proposal identifier occurs more than once.
    #[error("NNS proposal activity received duplicate proposal id {proposal_id}")]
    DuplicateProposalId {
        /// Repeated proposal identifier.
        proposal_id: u64,
    },

    /// A supplied proposal has no meaningful creation timestamp.
    #[error("NNS proposal {proposal_id} has proposal timestamp zero")]
    ZeroProposalTimestamp {
        /// Proposal identifier attached to the zero timestamp.
        proposal_id: u64,
    },

    /// A row-count conversion or aggregate increment exceeded `u64`.
    #[error("NNS proposal activity accounting overflow while updating {field}")]
    AccountingOverflow {
        /// Count or conversion that exceeded its representation.
        field: &'static str,
    },
}

/// Build one deterministic activity report from a complete caller-retained proposal collection.
pub fn build_nns_proposal_activity_report(
    request: &NnsProposalActivityRequest,
    collection: &NnsProposalCollectionState,
    proposals: &[NnsProposalRow],
) -> Result<NnsProposalActivityReport, NnsProposalActivityError> {
    validate_collection_state(collection).map_err(|error| {
        NnsProposalActivityError::InvalidCollectionState {
            reason: error.to_string(),
        }
    })?;
    if !collection.is_complete() {
        return Err(NnsProposalActivityError::CollectionNotComplete {
            status: collection.status(),
        });
    }
    validate_time_window(request)?;

    let expected = collection.proposals_fetched();
    let actual = u64::try_from(proposals.len()).map_err(|_| {
        NnsProposalActivityError::AccountingOverflow {
            field: "supplied_proposal_count",
        }
    })?;
    if actual != expected {
        return Err(NnsProposalActivityError::ProposalCountMismatch { expected, actual });
    }

    let mut proposal_ids = HashSet::with_capacity(proposals.len());
    aggregate_activity(request, collection, proposals, |proposal| {
        validate_proposal_row(proposal, &mut proposal_ids)
    })
}

#[cfg(feature = "nns-host")]
pub(super) fn build_activity_from_cache(
    request: &NnsProposalActivityRequest,
    cache: &super::cache::AdmittedProposalCache,
) -> Result<NnsProposalActivityReport, NnsProposalActivityError> {
    // Cache admission already establishes complete state, row accounting and unique positive ids.
    aggregate_activity(
        request,
        cache.collection_state(),
        cache.rows(),
        |proposal| {
            let proposal_id = proposal
                .proposal_id
                .ok_or(NnsProposalActivityError::MissingProposalId)?;
            validate_proposal_timestamp(proposal, proposal_id)
        },
    )
}

fn aggregate_activity(
    request: &NnsProposalActivityRequest,
    collection: &NnsProposalCollectionState,
    proposals: &[NnsProposalRow],
    mut validate_row: impl FnMut(&NnsProposalRow) -> Result<(), NnsProposalActivityError>,
) -> Result<NnsProposalActivityReport, NnsProposalActivityError> {
    let mut activity = ActivityAccumulator::default();
    for proposal in proposals {
        validate_row(proposal)?;
        activity.observe(request, proposal)?;
    }
    let source = collection.source().cloned().ok_or_else(|| {
        NnsProposalActivityError::InvalidCollectionState {
            reason: "complete collection has no concrete source provenance".to_string(),
        }
    })?;
    Ok(activity.into_report(request, collection, source))
}

#[derive(Default)]
struct ActivityAccumulator {
    topic_counts: BTreeMap<i32, u64>,
    status_counts: BTreeMap<i32, u64>,
    reward_status_counts: BTreeMap<i32, u64>,
    day_counts: BTreeMap<u64, u64>,
    included_proposal_count: u64,
    excluded_before_from_count: u64,
    excluded_at_or_after_until_count: u64,
    included_timestamp_bounds: Option<(u64, u64)>,
}

impl ActivityAccumulator {
    fn observe(
        &mut self,
        request: &NnsProposalActivityRequest,
        proposal: &NnsProposalRow,
    ) -> Result<(), NnsProposalActivityError> {
        let timestamp = proposal.proposal_timestamp_seconds;
        if request
            .from_proposal_timestamp_seconds
            .is_some_and(|from| timestamp < from)
        {
            return increment_count(
                &mut self.excluded_before_from_count,
                "excluded_before_from_count",
            );
        }
        if request
            .until_proposal_timestamp_seconds
            .is_some_and(|until| timestamp >= until)
        {
            return increment_count(
                &mut self.excluded_at_or_after_until_count,
                "excluded_at_or_after_until_count",
            );
        }

        increment_count(&mut self.included_proposal_count, "included_proposal_count")?;
        increment_count(
            self.topic_counts.entry(proposal.topic).or_default(),
            "topic_count",
        )?;
        increment_count(
            self.status_counts.entry(proposal.status).or_default(),
            "status_count",
        )?;
        increment_count(
            self.reward_status_counts
                .entry(proposal.reward_status)
                .or_default(),
            "reward_status_count",
        )?;
        let day_start = timestamp - (timestamp % SECONDS_PER_DAY);
        increment_count(self.day_counts.entry(day_start).or_default(), "day_count")?;
        self.included_timestamp_bounds = Some(match self.included_timestamp_bounds {
            Some((earliest, latest)) => (earliest.min(timestamp), latest.max(timestamp)),
            None => (timestamp, timestamp),
        });
        Ok(())
    }

    fn into_report(
        self,
        request: &NnsProposalActivityRequest,
        collection: &NnsProposalCollectionState,
        source: NnsGovernanceSourceProvenance,
    ) -> NnsProposalActivityReport {
        NnsProposalActivityReport {
            context: crate::nns::governance::NnsGovernanceAnalyticsContext {
                schema_version: NNS_PROPOSAL_ACTIVITY_REPORT_SCHEMA_VERSION,
                network: collection.network().to_string(),
                governance_canister_id: collection.governance_canister_id().to_string(),
                source,
                collection_started_at: collection.started_at().to_string(),
                collection_updated_at: collection.updated_at().to_string(),
                collection_page_count: collection.pages_fetched(),
                point_in_time_guaranteed: false,
            },
            collected_proposal_count: collection.proposals_fetched(),
            from_proposal_timestamp_seconds: request.from_proposal_timestamp_seconds,
            until_proposal_timestamp_seconds: request.until_proposal_timestamp_seconds,
            included_proposal_count: self.included_proposal_count,
            excluded_before_from_count: self.excluded_before_from_count,
            excluded_at_or_after_until_count: self.excluded_at_or_after_until_count,
            earliest_included_proposal_timestamp_seconds: self
                .included_timestamp_bounds
                .map(|bounds| bounds.0),
            latest_included_proposal_timestamp_seconds: self
                .included_timestamp_bounds
                .map(|bounds| bounds.1),
            topic_counts: self
                .topic_counts
                .into_iter()
                .map(|(topic, proposal_count)| NnsProposalTopicCount {
                    topic,

                    proposal_count,
                })
                .collect(),
            status_counts: self
                .status_counts
                .into_iter()
                .map(|(status, proposal_count)| NnsProposalStatusCount {
                    status,

                    proposal_count,
                })
                .collect(),
            reward_status_counts: self
                .reward_status_counts
                .into_iter()
                .map(
                    |(reward_status, proposal_count)| NnsProposalRewardStatusCount {
                        reward_status,

                        proposal_count,
                    },
                )
                .collect(),
            day_counts: self
                .day_counts
                .into_iter()
                .map(
                    |(day_start_timestamp_seconds, proposal_count)| NnsProposalDayCount {
                        day_start_timestamp_seconds,
                        proposal_count,
                    },
                )
                .collect(),
        }
    }
}

pub(super) const fn validate_time_window(
    request: &NnsProposalActivityRequest,
) -> Result<(), NnsProposalActivityError> {
    if let (Some(from), Some(until)) = (
        request.from_proposal_timestamp_seconds,
        request.until_proposal_timestamp_seconds,
    ) && from >= until
    {
        return Err(NnsProposalActivityError::InvalidTimeWindow {
            from_proposal_timestamp_seconds: from,
            until_proposal_timestamp_seconds: until,
        });
    }
    Ok(())
}

fn validate_proposal_row(
    proposal: &NnsProposalRow,
    proposal_ids: &mut HashSet<u64>,
) -> Result<(), NnsProposalActivityError> {
    let proposal_id = proposal
        .proposal_id
        .ok_or(NnsProposalActivityError::MissingProposalId)?;
    if proposal_id == 0 {
        return Err(NnsProposalActivityError::ZeroProposalId);
    }
    if !proposal_ids.insert(proposal_id) {
        return Err(NnsProposalActivityError::DuplicateProposalId { proposal_id });
    }
    validate_proposal_timestamp(proposal, proposal_id)
}

const fn validate_proposal_timestamp(
    proposal: &NnsProposalRow,
    proposal_id: u64,
) -> Result<(), NnsProposalActivityError> {
    if proposal.proposal_timestamp_seconds == 0 {
        return Err(NnsProposalActivityError::ZeroProposalTimestamp { proposal_id });
    }

    Ok(())
}

fn increment_count(count: &mut u64, field: &'static str) -> Result<(), NnsProposalActivityError> {
    *count = count
        .checked_add(1)
        .ok_or(NnsProposalActivityError::AccountingOverflow { field })?;
    Ok(())
}

#[cfg(test)]
mod tests;
