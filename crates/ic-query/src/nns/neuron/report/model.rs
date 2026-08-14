//! Module: nns::neuron::report::model
//!
//! Responsibility: define public NNS neuron request and report models.
//! Does not own: live transport, cache IO, or text rendering.
//! Boundary: preserves the unauthenticated Governance `NeuronInfo` fields without private state.

use super::classification::{NnsNeuronState, NnsNeuronType, NnsNeuronVisibility, NnsNeuronVote};
use crate::nns::governance::{NnsGovernanceReportContext, NnsGovernanceRequest};
use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// NnsKnownNeuronData
///
/// Public metadata attached to a registered known neuron.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsKnownNeuronData {
    /// Registered neuron name.
    pub name: String,
    /// Optional registered description.
    pub description: Option<String>,
    /// Registered related links.
    pub links: Vec<String>,
}

///
/// NnsNeuronBallotRow
///
/// One recent public ballot exposed by the Governance neuron index.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsNeuronBallotRow {
    /// Proposal identifier when supplied by Governance.
    pub proposal_id: Option<u64>,
    /// Raw Governance vote discriminant.
    pub vote: i32,
    /// Typed classification and stable display label for the raw vote.
    pub vote_text: NnsNeuronVote,
}

///
/// NnsNeuronRow
///
/// Public limited view of one NNS neuron returned by Governance.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsNeuronRow {
    /// Stable Governance neuron identifier.
    pub neuron_id: u64,
    /// Raw Governance state discriminant.
    pub state: i32,
    /// Typed classification and stable display label for the raw state.
    pub state_text: NnsNeuronState,
    /// Raw optional neuron visibility discriminant.
    pub visibility: Option<i32>,
    /// Typed classification and stable display label for the raw visibility.
    pub visibility_text: NnsNeuronVisibility,
    /// Raw optional neuron-type discriminant.
    pub neuron_type: Option<i32>,
    /// Typed classification and stable display label for the raw neuron type.
    pub neuron_type_text: NnsNeuronType,
    /// Public effective stake, including staked maturity, in e8s.
    pub stake_e8s: u64,
    /// Staked maturity included in effective stake, in e8s when supplied.
    pub staked_maturity_e8s_equivalent: Option<u64>,
    /// Current dissolve delay in seconds.
    pub dissolve_delay_seconds: u64,
    /// Current neuron age in seconds.
    pub age_seconds: u64,
    /// Neuron creation timestamp in Unix seconds.
    pub created_timestamp_seconds: u64,
    /// Governance retrieval timestamp in Unix seconds.
    pub retrieved_at_timestamp_seconds: u64,
    /// Deprecated Governance voting-power field retained losslessly.
    pub voting_power: u64,
    /// Current deciding voting power when supplied.
    pub deciding_voting_power: Option<u64>,
    /// Current potential voting power when supplied.
    pub potential_voting_power: Option<u64>,
    /// Last voting-power refresh timestamp in Unix seconds.
    pub voting_power_refreshed_timestamp_seconds: Option<u64>,
    /// Neurons' Fund join timestamp in Unix seconds when publicly visible.
    pub joined_community_fund_timestamp_seconds: Option<u64>,
    /// Eight-year dissolve-delay bonus base in e8s when supplied.
    pub eight_year_gang_bonus_base_e8s: Option<u64>,
    /// Registered public known-neuron metadata when present.
    pub known_neuron_data: Option<NnsKnownNeuronData>,
    /// Recent ballots visible to the unauthenticated caller.
    pub recent_ballots: Vec<NnsNeuronBallotRow>,
}

///
/// NnsNeuronListRequest
///
/// Request for one page of the public NNS Governance neuron index.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNeuronListRequest {
    /// Shared network, collection time, and source transport request.
    pub governance: NnsGovernanceRequest,
    /// Maximum rows to return.
    pub limit: u32,
    /// Exclusive lower neuron-id bound.
    pub exclusive_start_neuron_id: Option<u64>,
    /// Whether text output should include expanded metadata.
    pub verbose: bool,
}

impl NnsNeuronListRequest {
    /// Construct a first-page public neuron-index request.
    #[must_use]
    pub const fn new(governance: NnsGovernanceRequest, limit: u32) -> Self {
        Self {
            governance,
            limit,
            exclusive_start_neuron_id: None,
            verbose: false,
        }
    }

    /// Start strictly after the given neuron id.
    #[must_use]
    pub const fn with_exclusive_start_neuron_id(mut self, neuron_id: u64) -> Self {
        self.exclusive_start_neuron_id = Some(neuron_id);
        self
    }

    /// Select compact or expanded text rendering.
    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

///
/// NnsNeuronInfoRequest
///
/// Request for one public NNS Governance neuron view.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNeuronInfoRequest {
    /// Shared network, collection time, and source transport request.
    pub governance: NnsGovernanceRequest,
    /// Governance neuron identifier.
    pub neuron_id: u64,
    /// Whether text output should include expanded metadata.
    pub verbose: bool,
}

impl NnsNeuronInfoRequest {
    /// Construct a public neuron-detail request.
    #[must_use]
    pub const fn new(governance: NnsGovernanceRequest, neuron_id: u64) -> Self {
        Self {
            governance,
            neuron_id,
            verbose: false,
        }
    }

    /// Select compact or expanded text rendering.
    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

///
/// NnsNeuronListReport
///
/// Serializable page from the public NNS Governance neuron index.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsNeuronListReport {
    /// Shared Governance authority and transport provenance.
    #[serde(flatten)]
    pub context: NnsGovernanceReportContext,
    /// Cache path when the page came from a complete snapshot.
    pub cache_path: Option<String>,
    /// Whether rows came from a complete local snapshot.
    pub from_cache: bool,
    /// Requested page limit.
    pub requested_limit: u32,
    /// Exclusive lower neuron-id bound.
    pub exclusive_start_neuron_id: Option<u64>,
    /// Cursor for a possible next page.
    pub next_start_neuron_id: Option<u64>,
    /// Total rows in the complete snapshot when known.
    pub total_neuron_count: Option<usize>,
    /// Whether all returned rows are guaranteed to describe one Governance instant.
    pub point_in_time_guaranteed: bool,
    /// Number of rows returned in this view.
    pub returned_neuron_count: usize,
    /// Whether verbose text rendering was requested.
    pub verbose: bool,
    /// Canonically ascending neuron rows.
    pub neurons: Vec<NnsNeuronRow>,
}

///
/// NnsNeuronInfoReport
///
/// Serializable public view of one NNS neuron.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsNeuronInfoReport {
    /// Shared Governance authority and transport provenance.
    #[serde(flatten)]
    pub context: NnsGovernanceReportContext,
    /// Cache path when the row came from a complete snapshot.
    pub cache_path: Option<String>,
    /// Whether the row came from a complete local snapshot.
    pub from_cache: bool,
    /// Whether verbose text rendering was requested.
    pub verbose: bool,
    /// Public Governance neuron view.
    pub neuron: NnsNeuronRow,
}
