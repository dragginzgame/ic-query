//! Module: nns::proposals::report::model::reports
//!
//! Responsibility: serialized NNS proposal report and row DTOs.
//! Does not own: request selection, source transport, or text rendering.
//! Boundary: defines the stable JSON contract for NNS proposal output.

use super::selection::{
    NnsProposalRewardStatus, NnsProposalStatus, NnsProposalTopic, NnsProposalVote,
};
use crate::{
    nns::governance::NnsGovernanceReportContext,
    report::{ReportDataSource, ReportResultScope},
};
use serde::{Deserialize, Serialize};

///
/// NnsProposalListReport
///
/// Serializable report for a bounded NNS governance proposal listing.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalListReport {
    /// Shared Governance authority and transport provenance.
    #[serde(flatten)]
    pub context: NnsGovernanceReportContext,
    /// Whether the rows came from a live call or a local complete snapshot.
    pub data_source: ReportDataSource,
    /// Snapshot file used for this report, absent for a live result.
    pub cache_path: Option<String>,
    /// Snapshot completeness when a cache supplied the rows; absent for live results.
    pub cache_complete: Option<bool>,
    /// Maximum number of matching rows requested by the caller.
    pub requested_limit: u32,
    /// Exclusive upper proposal-id bound; absent on the first page.
    pub before_proposal_id: Option<u64>,
    /// Canonical decision-status filter label.
    pub status_filter: String,
    /// Canonical reward-settlement filter label.
    pub reward_status_filter: String,
    /// Canonical proposal-topic filter label.
    pub topic_filter: String,
    /// Required proposer neuron id, when selected.
    pub proposer_filter: Option<u64>,
    /// Caller-selected case-insensitive search over title, summary, action, and URL.
    pub query_filter: Option<String>,
    /// Canonical sort key used to order the returned rows.
    pub sort: String,
    /// Local sort direction, or `none` when preserving API order.
    pub sort_direction: String,
    /// Whether filtering and sorting cover a fetched page or a complete collection.
    pub result_scope: ReportResultScope,
    /// Whether the text renderer includes expanded proposal fields.
    pub verbose: bool,
    /// Number of proposal rows returned in this report.
    pub proposal_count: usize,
    /// Proposal rows in the selected result order.
    pub proposals: Vec<NnsProposalRow>,
}

///
/// NnsProposalReport
///
/// Serializable report for one NNS governance proposal detail lookup.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalReport {
    /// Shared Governance authority and transport provenance.
    #[serde(flatten)]
    pub context: NnsGovernanceReportContext,
    /// Whether the rows came from a live call or a local complete snapshot.
    pub data_source: ReportDataSource,
    /// Snapshot file used for this report, absent for a live result.
    pub cache_path: Option<String>,
    /// Snapshot completeness when a cache supplied the rows; absent for live results.
    pub cache_complete: Option<bool>,
    /// Governance proposal identifier.
    pub proposal_id: u64,
    /// Whether the human-facing detail renderer displays ballot rows.
    pub show_ballots: bool,
    /// Whether the text renderer includes expanded proposal fields.
    pub verbose: bool,
    /// Exact proposal row returned for the requested identifier.
    pub proposal: NnsProposalRow,
}

///
/// NnsProposalRow
///
/// Serializable row for one NNS governance proposal.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalRow {
    /// Governance proposal identifier, when supplied in the source row.
    pub proposal_id: Option<u64>,
    /// Neuron that submitted the proposal, when supplied by Governance.
    pub proposer_neuron_id: Option<u64>,
    /// Raw native Governance topic code.
    pub topic: i32,
    /// Classification derived from the raw topic code.
    pub topic_text: NnsProposalTopic,
    /// Raw native Governance decision-status code.
    pub status: i32,
    /// Classification derived from the raw decision-status code.
    pub status_text: NnsProposalStatus,
    /// Raw native Governance reward-settlement code.
    pub reward_status: i32,
    /// Classification derived from the raw reward-settlement code.
    pub reward_status_text: NnsProposalRewardStatus,
    /// Proposal title, when supplied by Governance.
    pub title: Option<String>,
    /// Proposal summary supplied by Governance.
    pub summary: String,
    /// Proposal information URL supplied by Governance.
    pub url: String,
    /// Human-readable action classification, when an action is present.
    pub action_text: Option<String>,
    /// Stake charged for a rejected proposal, in ICP e8s.
    pub reject_cost_e8s: u64,
    /// Proposal creation time in Unix seconds.
    pub proposal_timestamp_seconds: u64,
    /// UTC text derived from the proposal creation timestamp.
    pub proposed_at: String,
    /// Voting deadline in Unix seconds, when supplied by Governance.
    pub deadline_timestamp_seconds: Option<u64>,
    /// UTC text derived from a nonzero voting deadline.
    pub deadline_at: Option<String>,
    /// Decision time in Unix seconds; zero means no decision time was recorded.
    pub decided_timestamp_seconds: u64,
    /// UTC decision time, absent when the raw timestamp is zero.
    pub decided_at: Option<String>,
    /// Execution time in Unix seconds; zero means no execution time was recorded.
    pub executed_timestamp_seconds: u64,
    /// UTC execution time, absent when the raw timestamp is zero.
    pub executed_at: Option<String>,
    /// Execution failure time in Unix seconds; zero means no failure time was recorded.
    pub failed_timestamp_seconds: u64,
    /// UTC failure time, absent when the raw timestamp is zero.
    pub failed_at: Option<String>,
    /// Native reward distribution round assigned to the proposal.
    pub reward_event_round: u64,
    /// Total potential voting power, when supplied by Governance.
    pub total_potential_voting_power: Option<u64>,
    /// Most recent vote tally, when supplied by Governance.
    pub latest_tally: Option<NnsProposalTally>,
    /// Number of ballot rows returned by Governance.
    pub ballot_count: usize,
    /// Ballots returned by Governance, ordered by neuron id.
    pub ballots: Vec<NnsProposalBallotRow>,
}

///
/// NnsProposalBallotRow
///
/// Serializable NNS proposal ballot row.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalBallotRow {
    /// Governance neuron identifier attached to this ballot.
    pub neuron_id: u64,
    /// Raw native Governance vote code.
    pub vote: i32,
    /// Classification derived from the raw vote code.
    pub vote_text: NnsProposalVote,
    /// Voting power recorded for this neuron ballot.
    pub voting_power: u64,
}

///
/// NnsProposalTally
///
/// Serializable NNS proposal vote tally.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalTally {
    /// Time of the tally in Unix seconds.
    pub timestamp_seconds: u64,
    /// Total voting power cast in affirmative ballots.
    pub yes: u64,
    /// Total voting power cast in negative ballots.
    pub no: u64,
    /// Total voting power represented by Governance for this tally.
    pub total: u64,
}
