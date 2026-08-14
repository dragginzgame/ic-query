//! Module: nns::neuron::report::wire
//!
//! Responsibility: Candid wire types for NNS Governance neuron queries.
//! Does not own: live transport, report DTOs, or text rendering.
//! Boundary: models only the fields needed by bounded neuron reports.

use candid::{CandidType, Deserialize};

///
/// GetNeuronIndexRequestWire
///
/// Candid request for one bounded public neuron-index page.
///

#[derive(CandidType)]
pub(in crate::nns::neuron::report) struct GetNeuronIndexRequestWire {
    pub(in crate::nns::neuron::report) exclusive_start_neuron_id: Option<NeuronIdWire>,
    pub(in crate::nns::neuron::report) page_size: Option<u32>,
}

///
/// GetNeuronIndexResultWire
///
/// Candid result returned by `get_neuron_index`.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) enum GetNeuronIndexResultWire {
    Ok(NeuronIndexDataWire),
    Err(GovernanceErrorWire),
}

impl GovernanceResult<NeuronIndexDataWire> for GetNeuronIndexResultWire {
    fn into_result(self) -> Result<NeuronIndexDataWire, GovernanceErrorWire> {
        match self {
            Self::Ok(data) => Ok(data),
            Self::Err(error) => Err(error),
        }
    }
}

///
/// GetNeuronInfoResultWire
///
/// Candid result returned by `get_neuron_info`.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) enum GetNeuronInfoResultWire {
    Ok(Box<NeuronInfoWire>),
    Err(GovernanceErrorWire),
}

impl GovernanceResult<NeuronInfoWire> for GetNeuronInfoResultWire {
    fn into_result(self) -> Result<NeuronInfoWire, GovernanceErrorWire> {
        match self {
            Self::Ok(data) => Ok(*data),
            Self::Err(error) => Err(error),
        }
    }
}

///
/// GovernanceResult
///
/// Shared conversion boundary for Governance result variants.
///

pub(in crate::nns::neuron::report) trait GovernanceResult<Response> {
    fn into_result(self) -> Result<Response, GovernanceErrorWire>;
}

///
/// NeuronIndexDataWire
///
/// Candid bounded neuron-index payload.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) struct NeuronIndexDataWire {
    pub(in crate::nns::neuron::report) neurons: Vec<NeuronInfoWire>,
}

///
/// GovernanceErrorWire
///
/// Candid Governance application-level failure.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) struct GovernanceErrorWire {
    pub(in crate::nns::neuron::report) error_message: String,
    pub(in crate::nns::neuron::report) error_type: i32,
}

///
/// NeuronInfoWire
///
/// Candid public limited neuron view returned by Governance.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) struct NeuronInfoWire {
    pub(in crate::nns::neuron::report) id: Option<NeuronIdWire>,
    pub(in crate::nns::neuron::report) dissolve_delay_seconds: u64,
    pub(in crate::nns::neuron::report) recent_ballots: Vec<BallotInfoWire>,
    pub(in crate::nns::neuron::report) neuron_type: Option<i32>,
    pub(in crate::nns::neuron::report) created_timestamp_seconds: u64,
    pub(in crate::nns::neuron::report) state: i32,
    pub(in crate::nns::neuron::report) stake_e8s: u64,
    pub(in crate::nns::neuron::report) joined_community_fund_timestamp_seconds: Option<u64>,
    pub(in crate::nns::neuron::report) retrieved_at_timestamp_seconds: u64,
    pub(in crate::nns::neuron::report) visibility: Option<i32>,
    pub(in crate::nns::neuron::report) known_neuron_data: Option<KnownNeuronDataWire>,
    pub(in crate::nns::neuron::report) age_seconds: u64,
    pub(in crate::nns::neuron::report) voting_power: u64,
    pub(in crate::nns::neuron::report) voting_power_refreshed_timestamp_seconds: Option<u64>,
    pub(in crate::nns::neuron::report) deciding_voting_power: Option<u64>,
    pub(in crate::nns::neuron::report) potential_voting_power: Option<u64>,
    pub(in crate::nns::neuron::report) eight_year_gang_bonus_base_e8s: Option<u64>,
    pub(in crate::nns::neuron::report) staked_maturity_e8s_equivalent: Option<u64>,
}

///
/// NeuronIdWire
///
/// Candid NNS Governance neuron identifier.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) struct NeuronIdWire {
    pub(in crate::nns::neuron::report) id: u64,
}

///
/// ProposalIdWire
///
/// Candid proposal identifier attached to a recent neuron ballot.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) struct ProposalIdWire {
    pub(in crate::nns::neuron::report) id: u64,
}

///
/// BallotInfoWire
///
/// Candid recent ballot exposed by a public neuron view.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) struct BallotInfoWire {
    pub(in crate::nns::neuron::report) vote: i32,
    pub(in crate::nns::neuron::report) proposal_id: Option<ProposalIdWire>,
}

///
/// KnownNeuronDataWire
///
/// Candid registered known-neuron metadata.
///

#[derive(CandidType, Deserialize)]
pub(in crate::nns::neuron::report) struct KnownNeuronDataWire {
    pub(in crate::nns::neuron::report) name: String,
    pub(in crate::nns::neuron::report) description: Option<String>,
    pub(in crate::nns::neuron::report) links: Option<Vec<String>>,
}
