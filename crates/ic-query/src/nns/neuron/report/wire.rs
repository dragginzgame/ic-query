//! Module: nns::neuron::report::wire
//!
//! Responsibility: Candid wire types for NNS Governance neuron queries.
//! Does not own: live transport, report DTOs, or text rendering.
//! Boundary: models only the fields needed by bounded neuron reports.

use crate::nns::governance::{GovernanceError, NnsGovernanceNeuronId, NnsGovernanceProposalId};
use ic_platform::candid::contract::{CandidType, serde::Deserialize};

///
/// GetNeuronIndexRequestWire
///
/// Candid request for one bounded public neuron-index page.
///

#[derive(CandidType)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::nns::neuron::report) struct GetNeuronIndexRequestWire {
    pub(in crate::nns::neuron::report) exclusive_start_neuron_id: Option<NnsGovernanceNeuronId>,
    pub(in crate::nns::neuron::report) page_size: Option<u32>,
}

///
/// GetNeuronIndexResultWire
///
/// Candid result returned by `get_neuron_index`.
///

pub(in crate::nns::neuron::report) type GetNeuronIndexResultWire =
    Result<NeuronIndexDataWire, GovernanceError>;

///
/// GetNeuronInfoResultWire
///
/// Candid result returned by `get_neuron_info`.
///

pub(in crate::nns::neuron::report) type GetNeuronInfoResultWire =
    Result<Box<NeuronInfoWire>, GovernanceError>;

///
/// NeuronIndexDataWire
///
/// Candid bounded neuron-index payload.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::nns::neuron::report) struct NeuronIndexDataWire {
    pub(in crate::nns::neuron::report) neurons: Vec<NeuronInfoWire>,
}

///
/// NeuronInfoWire
///
/// Candid public limited neuron view returned by Governance.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::nns::neuron::report) struct NeuronInfoWire {
    pub(in crate::nns::neuron::report) id: Option<NnsGovernanceNeuronId>,
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
/// BallotInfoWire
///
/// Candid recent ballot exposed by a public neuron view.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::nns::neuron::report) struct BallotInfoWire {
    pub(in crate::nns::neuron::report) vote: i32,
    pub(in crate::nns::neuron::report) proposal_id: Option<NnsGovernanceProposalId>,
}

///
/// KnownNeuronDataWire
///
/// Candid registered known-neuron metadata.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::nns::neuron::report) struct KnownNeuronDataWire {
    pub(in crate::nns::neuron::report) name: String,
    pub(in crate::nns::neuron::report) description: Option<String>,
    pub(in crate::nns::neuron::report) links: Option<Vec<String>>,
}

#[cfg(all(test, feature = "nns-host"))]
mod tests {
    use super::*;
    use crate::{
        candid_decode::decode_reply,
        nns::neuron::report::{NnsNeuronError, source::governance_result},
    };
    use ic_platform::candid::contract::encode_one;

    // Governance's external variant labels are part of the Candid contract.
    #[derive(CandidType)]
    #[candid_path("ic_platform::candid::contract")]
    enum GovernanceReply<T> {
        Ok(T),
        Err(GovernanceError),
    }

    #[test]
    fn index_reply_accepts_the_native_ok_variant() {
        let bytes = encode_one(GovernanceReply::Ok(NeuronIndexDataWire {
            neurons: Vec::new(),
        }))
        .unwrap();
        let reply = decode_reply::<GetNeuronIndexResultWire>(&bytes).unwrap();
        let page = governance_result(reply).unwrap();
        assert!(page.neurons.is_empty());
    }

    #[test]
    fn boxed_info_reply_projects_the_native_error_variant() {
        let bytes = encode_one(GovernanceReply::<Box<NeuronInfoWire>>::Err(
            GovernanceError {
                error_type: 4,
                error_message: "fixture neuron is absent".to_string(),
            },
        ))
        .unwrap();
        let reply = decode_reply::<GetNeuronInfoResultWire>(&bytes).unwrap();
        assert!(matches!(
            governance_result(reply),
            Err(NnsNeuronError::GovernanceResponse { error_type: 4, message })
                if message == "fixture neuron is absent"
        ));
    }
}
