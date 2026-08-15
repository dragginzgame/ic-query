//! Module: nns::proposals::report::activity
//!
//! Responsibility: validate and aggregate complete caller-retained NNS proposal collections.
//! Does not own: collection transport, persistence, cache policy, or process output.
//! Boundary: projects complete proposal evidence into deterministic portable activity reports.

use super::{
    NnsProposalCollectionState, NnsProposalCollectionStatus,
    collection::validate_collection_state,
    model::{NnsProposalRewardStatus, NnsProposalRow, NnsProposalStatus, NnsProposalTopic},
};
use crate::{
    nns::{
        MAINNET_GOVERNANCE_CANISTER_ID,
        governance::{NnsGovernanceSourceProvenance, validate_governance_report_source},
    },
    subnet_catalog::MAINNET_NETWORK,
};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalTopicCount {
    /// Raw native Governance topic code.
    pub topic: i32,
    /// Classification derived from the raw topic code.
    pub topic_text: NnsProposalTopic,
    /// Number of included proposals with this topic code.
    pub proposal_count: u64,
}

///
/// NnsProposalStatusCount
///
/// Proposal count for one raw native Governance decision-status code.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalStatusCount {
    /// Raw native Governance decision-status code.
    pub status: i32,
    /// Classification derived from the raw status code.
    pub status_text: NnsProposalStatus,
    /// Number of included proposals with this status code.
    pub proposal_count: u64,
}

///
/// NnsProposalRewardStatusCount
///
/// Proposal count for one raw native Governance reward-status code.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalRewardStatusCount {
    /// Raw native Governance reward-status code.
    pub reward_status: i32,
    /// Classification derived from the raw reward-status code.
    pub reward_status_text: NnsProposalRewardStatus,
    /// Number of included proposals with this reward-status code.
    pub proposal_count: u64,
}

///
/// NnsProposalDayCount
///
/// Proposal count for one UTC proposal-creation day.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalActivityReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Network identity retained by the complete collection.
    pub network: String,
    /// Fixed NNS Governance canister principal retained by the collection.
    pub governance_canister_id: String,
    /// Concrete transport and collector provenance retained by the collection.
    pub source: NnsGovernanceSourceProvenance,
    /// Caller-supplied time attached to collection creation.
    pub collection_started_at: String,
    /// Caller-supplied time attached to the final admitted page.
    pub collection_updated_at: String,
    /// Number of admitted source pages in the complete collection.
    pub collection_page_count: u32,
    /// Number of admitted proposal rows in the complete collection.
    pub collected_proposal_count: u64,
    /// Whether the sequential collection is guaranteed to represent one point in time.
    pub point_in_time_guaranteed: bool,
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
/// NnsProposalActivityValidationError
///
/// Pure validation failure for an untrusted serialized or in-memory activity report.
///

#[derive(Debug, Eq, PartialEq, ThisError)]
#[error("invalid NNS proposal activity report: {reason}")]
pub struct NnsProposalActivityValidationError {
    /// Deterministic invariant failure.
    pub reason: String,
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
        status: NnsProposalCollectionStatus,
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

    /// A typed topic classification disagrees with its raw code.
    #[error(
        "NNS proposal {proposal_id} topic classification {actual:?} does not match raw code {topic} ({expected:?})"
    )]
    TopicClassificationMismatch {
        /// Proposal identifier carrying the mismatch.
        proposal_id: u64,
        /// Raw native topic code.
        topic: i32,
        /// Classification supplied by the row.
        actual: NnsProposalTopic,
        /// Classification derived from the raw code.
        expected: NnsProposalTopic,
    },

    /// A typed decision-status classification disagrees with its raw code.
    #[error(
        "NNS proposal {proposal_id} status classification {actual:?} does not match raw code {status} ({expected:?})"
    )]
    StatusClassificationMismatch {
        /// Proposal identifier carrying the mismatch.
        proposal_id: u64,
        /// Raw native decision-status code.
        status: i32,
        /// Classification supplied by the row.
        actual: NnsProposalStatus,
        /// Classification derived from the raw code.
        expected: NnsProposalStatus,
    },

    /// A typed reward-status classification disagrees with its raw code.
    #[error(
        "NNS proposal {proposal_id} reward-status classification {actual:?} does not match raw code {reward_status} ({expected:?})"
    )]
    RewardStatusClassificationMismatch {
        /// Proposal identifier carrying the mismatch.
        proposal_id: u64,
        /// Raw native reward-status code.
        reward_status: i32,
        /// Classification supplied by the row.
        actual: NnsProposalRewardStatus,
        /// Classification derived from the raw code.
        expected: NnsProposalRewardStatus,
    },

    /// A row-count conversion or aggregate increment exceeded `u64`.
    #[error("NNS proposal activity accounting overflow while updating {field}")]
    AccountingOverflow {
        /// Count or conversion that exceeded its representation.
        field: &'static str,
    },

    /// The projected report failed its shared publication invariants.
    #[error(transparent)]
    InvalidReport(#[from] NnsProposalActivityValidationError),
}

/// Validate every activity-report invariant available without source rows or live host calls.
pub fn validate_nns_proposal_activity_report(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    validate_activity_header(report)?;
    validate_activity_selection(report)?;
    validate_topic_counts(report)?;
    validate_status_counts(report)?;
    validate_reward_status_counts(report)?;
    validate_day_counts(report)
}

fn validate_activity_header(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    if report.schema_version != NNS_PROPOSAL_ACTIVITY_REPORT_SCHEMA_VERSION {
        return Err(invalid_validation(format!(
            "schema version {} does not equal {}",
            report.schema_version, NNS_PROPOSAL_ACTIVITY_REPORT_SCHEMA_VERSION
        )));
    }
    if report.network != MAINNET_NETWORK {
        return Err(invalid_validation(format!(
            "network is {}, expected {MAINNET_NETWORK}",
            report.network
        )));
    }
    if report.governance_canister_id != MAINNET_GOVERNANCE_CANISTER_ID {
        return Err(invalid_validation(format!(
            "governance_canister_id is {}, expected {MAINNET_GOVERNANCE_CANISTER_ID}",
            report.governance_canister_id
        )));
    }
    if report.collection_page_count == 0 {
        return Err(invalid_validation(
            "complete activity report must retain at least one collection page",
        ));
    }
    if report.point_in_time_guaranteed {
        return Err(invalid_validation(
            "sequential proposal activity cannot claim a point-in-time snapshot",
        ));
    }

    validate_governance_report_source(&report.network, &report.source).map_err(|error| {
        let context = match &report.source {
            NnsGovernanceSourceProvenance::ReplicaQuery { .. } => "source",
            NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall { .. } => "provenance",
        };
        invalid_validation(format!("invalid collection {context}: {error}"))
    })
}

fn validate_activity_selection(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    if let (Some(from), Some(until)) = (
        report.from_proposal_timestamp_seconds,
        report.until_proposal_timestamp_seconds,
    ) && from >= until
    {
        return Err(invalid_validation(format!(
            "from proposal timestamp {from} must be below until timestamp {until}"
        )));
    }
    if report.from_proposal_timestamp_seconds.is_none() && report.excluded_before_from_count != 0 {
        return Err(invalid_validation(
            "excluded_before_from_count must be zero without a lower bound",
        ));
    }
    if report.until_proposal_timestamp_seconds.is_none()
        && report.excluded_at_or_after_until_count != 0
    {
        return Err(invalid_validation(
            "excluded_at_or_after_until_count must be zero without an upper bound",
        ));
    }

    let accounted = report
        .included_proposal_count
        .checked_add(report.excluded_before_from_count)
        .and_then(|count| count.checked_add(report.excluded_at_or_after_until_count))
        .ok_or_else(|| invalid_validation("proposal selection count overflow"))?;
    if accounted != report.collected_proposal_count {
        return Err(invalid_validation(format!(
            "selection accounts for {accounted} proposals, expected {}",
            report.collected_proposal_count
        )));
    }
    validate_included_range(report)
}

fn validate_included_range(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    let (earliest, latest) = match (
        report.earliest_included_proposal_timestamp_seconds,
        report.latest_included_proposal_timestamp_seconds,
    ) {
        (None, None) if report.included_proposal_count == 0 => return Ok(()),
        (Some(earliest), Some(latest)) if report.included_proposal_count > 0 => (earliest, latest),
        _ => {
            return Err(invalid_validation(
                "included timestamp range presence disagrees with included_proposal_count",
            ));
        }
    };
    if earliest == 0 || earliest > latest {
        return Err(invalid_validation(
            "included proposal timestamps must be nonzero and ascending",
        ));
    }
    if report
        .from_proposal_timestamp_seconds
        .is_some_and(|from| earliest < from)
    {
        return Err(invalid_validation(
            "earliest included proposal timestamp precedes the lower bound",
        ));
    }
    if report
        .until_proposal_timestamp_seconds
        .is_some_and(|until| latest >= until)
    {
        return Err(invalid_validation(
            "latest included proposal timestamp reaches or exceeds the upper bound",
        ));
    }
    Ok(())
}

fn validate_topic_counts(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    let mut previous = None;
    let mut total = 0_u64;
    for row in &report.topic_counts {
        if previous.is_some_and(|topic| topic >= row.topic) {
            return Err(invalid_validation(
                "topic count rows are not strictly raw-code ordered",
            ));
        }
        if row.topic_text != NnsProposalTopic::from_code(row.topic) {
            return Err(invalid_validation(format!(
                "topic classification for raw code {} is inconsistent",
                row.topic
            )));
        }
        total = add_dimension_count(total, row.proposal_count, "topic")?;
        previous = Some(row.topic);
    }
    validate_dimension_total(total, report.included_proposal_count, "topic")
}

fn validate_status_counts(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    let mut previous = None;
    let mut total = 0_u64;
    for row in &report.status_counts {
        if previous.is_some_and(|status| status >= row.status) {
            return Err(invalid_validation(
                "status count rows are not strictly raw-code ordered",
            ));
        }
        if row.status_text != NnsProposalStatus::from_code(row.status) {
            return Err(invalid_validation(format!(
                "status classification for raw code {} is inconsistent",
                row.status
            )));
        }
        total = add_dimension_count(total, row.proposal_count, "status")?;
        previous = Some(row.status);
    }
    validate_dimension_total(total, report.included_proposal_count, "status")
}

fn validate_reward_status_counts(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    let mut previous = None;
    let mut total = 0_u64;
    for row in &report.reward_status_counts {
        if previous.is_some_and(|reward_status| reward_status >= row.reward_status) {
            return Err(invalid_validation(
                "reward-status count rows are not strictly raw-code ordered",
            ));
        }
        if row.reward_status_text != NnsProposalRewardStatus::from_code(row.reward_status) {
            return Err(invalid_validation(format!(
                "reward-status classification for raw code {} is inconsistent",
                row.reward_status
            )));
        }
        total = add_dimension_count(total, row.proposal_count, "reward-status")?;
        previous = Some(row.reward_status);
    }
    validate_dimension_total(total, report.included_proposal_count, "reward-status")
}

fn validate_day_counts(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    let mut previous = None;
    let mut total = 0_u64;
    for row in &report.day_counts {
        if row.day_start_timestamp_seconds % SECONDS_PER_DAY != 0 {
            return Err(invalid_validation(
                "day count row is not aligned to 00:00:00 UTC",
            ));
        }
        if previous.is_some_and(|day| day >= row.day_start_timestamp_seconds) {
            return Err(invalid_validation(
                "day count rows are not strictly time ordered",
            ));
        }
        total = add_dimension_count(total, row.proposal_count, "day")?;
        previous = Some(row.day_start_timestamp_seconds);
    }
    validate_dimension_total(total, report.included_proposal_count, "day")?;
    validate_day_range(report)
}

fn validate_day_range(
    report: &NnsProposalActivityReport,
) -> Result<(), NnsProposalActivityValidationError> {
    if report.included_proposal_count == 0 {
        return Ok(());
    }
    let (Some(earliest), Some(latest)) = (
        report.earliest_included_proposal_timestamp_seconds,
        report.latest_included_proposal_timestamp_seconds,
    ) else {
        return Err(invalid_validation(
            "included timestamp range is absent for positive day counts",
        ));
    };
    let expected_first = earliest - (earliest % SECONDS_PER_DAY);
    let expected_last = latest - (latest % SECONDS_PER_DAY);
    let (Some(first), Some(last)) = (report.day_counts.first(), report.day_counts.last()) else {
        return Err(invalid_validation(
            "positive included count requires nonempty day counts",
        ));
    };
    let first = first.day_start_timestamp_seconds;
    let last = last.day_start_timestamp_seconds;
    if first != expected_first || last != expected_last {
        return Err(invalid_validation(
            "day count endpoints do not cover the included timestamp range",
        ));
    }
    Ok(())
}

fn add_dimension_count(
    total: u64,
    count: u64,
    dimension: &'static str,
) -> Result<u64, NnsProposalActivityValidationError> {
    if count == 0 {
        return Err(invalid_validation(format!(
            "{dimension} count row must be nonzero"
        )));
    }
    total
        .checked_add(count)
        .ok_or_else(|| invalid_validation(format!("{dimension} count total overflow")))
}

fn validate_dimension_total(
    actual: u64,
    expected: u64,
    dimension: &'static str,
) -> Result<(), NnsProposalActivityValidationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(invalid_validation(format!(
            "{dimension} counts sum to {actual}, expected {expected}"
        )))
    }
}

fn invalid_validation(reason: impl Into<String>) -> NnsProposalActivityValidationError {
    NnsProposalActivityValidationError {
        reason: reason.into(),
    }
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

    let mut activity = ActivityAccumulator::with_capacity(proposals.len());
    for proposal in proposals {
        activity.observe(request, proposal)?;
    }

    let source = collection.source().cloned().ok_or_else(|| {
        NnsProposalActivityError::InvalidCollectionState {
            reason: "complete collection has no concrete source provenance".to_string(),
        }
    })?;
    let report = activity.into_report(request, collection, expected, source);
    validate_nns_proposal_activity_report(&report)?;
    Ok(report)
}

struct ActivityAccumulator {
    proposal_ids: HashSet<u64>,
    topic_counts: BTreeMap<i32, u64>,
    status_counts: BTreeMap<i32, u64>,
    reward_status_counts: BTreeMap<i32, u64>,
    day_counts: BTreeMap<u64, u64>,
    included_proposal_count: u64,
    excluded_before_from_count: u64,
    excluded_at_or_after_until_count: u64,
    earliest_included_proposal_timestamp_seconds: Option<u64>,
    latest_included_proposal_timestamp_seconds: Option<u64>,
}

impl ActivityAccumulator {
    fn with_capacity(proposal_count: usize) -> Self {
        Self {
            proposal_ids: HashSet::with_capacity(proposal_count),
            topic_counts: BTreeMap::new(),
            status_counts: BTreeMap::new(),
            reward_status_counts: BTreeMap::new(),
            day_counts: BTreeMap::new(),
            included_proposal_count: 0,
            excluded_before_from_count: 0,
            excluded_at_or_after_until_count: 0,
            earliest_included_proposal_timestamp_seconds: None,
            latest_included_proposal_timestamp_seconds: None,
        }
    }

    fn observe(
        &mut self,
        request: &NnsProposalActivityRequest,
        proposal: &NnsProposalRow,
    ) -> Result<(), NnsProposalActivityError> {
        validate_proposal_row(proposal, &mut self.proposal_ids)?;
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
        self.earliest_included_proposal_timestamp_seconds = Some(
            self.earliest_included_proposal_timestamp_seconds
                .map_or(timestamp, |earliest| earliest.min(timestamp)),
        );
        self.latest_included_proposal_timestamp_seconds = Some(
            self.latest_included_proposal_timestamp_seconds
                .map_or(timestamp, |latest| latest.max(timestamp)),
        );
        Ok(())
    }

    fn into_report(
        self,
        request: &NnsProposalActivityRequest,
        collection: &NnsProposalCollectionState,
        collected_proposal_count: u64,
        source: NnsGovernanceSourceProvenance,
    ) -> NnsProposalActivityReport {
        NnsProposalActivityReport {
            schema_version: NNS_PROPOSAL_ACTIVITY_REPORT_SCHEMA_VERSION,
            network: collection.network().to_string(),
            governance_canister_id: collection.governance_canister_id().to_string(),
            source,
            collection_started_at: collection.started_at().to_string(),
            collection_updated_at: collection.updated_at().to_string(),
            collection_page_count: collection.pages_fetched(),
            collected_proposal_count,
            point_in_time_guaranteed: false,
            from_proposal_timestamp_seconds: request.from_proposal_timestamp_seconds,
            until_proposal_timestamp_seconds: request.until_proposal_timestamp_seconds,
            included_proposal_count: self.included_proposal_count,
            excluded_before_from_count: self.excluded_before_from_count,
            excluded_at_or_after_until_count: self.excluded_at_or_after_until_count,
            earliest_included_proposal_timestamp_seconds: self
                .earliest_included_proposal_timestamp_seconds,
            latest_included_proposal_timestamp_seconds: self
                .latest_included_proposal_timestamp_seconds,
            topic_counts: self
                .topic_counts
                .into_iter()
                .map(|(topic, proposal_count)| NnsProposalTopicCount {
                    topic,
                    topic_text: NnsProposalTopic::from_code(topic),
                    proposal_count,
                })
                .collect(),
            status_counts: self
                .status_counts
                .into_iter()
                .map(|(status, proposal_count)| NnsProposalStatusCount {
                    status,
                    status_text: NnsProposalStatus::from_code(status),
                    proposal_count,
                })
                .collect(),
            reward_status_counts: self
                .reward_status_counts
                .into_iter()
                .map(
                    |(reward_status, proposal_count)| NnsProposalRewardStatusCount {
                        reward_status,
                        reward_status_text: NnsProposalRewardStatus::from_code(reward_status),
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

const fn validate_time_window(
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
    if proposal.proposal_timestamp_seconds == 0 {
        return Err(NnsProposalActivityError::ZeroProposalTimestamp { proposal_id });
    }

    let expected_topic = NnsProposalTopic::from_code(proposal.topic);
    if proposal.topic_text != expected_topic {
        return Err(NnsProposalActivityError::TopicClassificationMismatch {
            proposal_id,
            topic: proposal.topic,
            actual: proposal.topic_text,
            expected: expected_topic,
        });
    }
    let expected_status = NnsProposalStatus::from_code(proposal.status);
    if proposal.status_text != expected_status {
        return Err(NnsProposalActivityError::StatusClassificationMismatch {
            proposal_id,
            status: proposal.status,
            actual: proposal.status_text,
            expected: expected_status,
        });
    }
    let expected_reward_status = NnsProposalRewardStatus::from_code(proposal.reward_status);
    if proposal.reward_status_text != expected_reward_status {
        return Err(
            NnsProposalActivityError::RewardStatusClassificationMismatch {
                proposal_id,
                reward_status: proposal.reward_status,
                actual: proposal.reward_status_text,
                expected: expected_reward_status,
            },
        );
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
