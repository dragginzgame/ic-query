//! Module: nns::neuron::report::source::live
//!
//! Responsibility: query and project the public NNS Governance neuron index.
//! Does not own: source-independent pagination validation, report assembly, or cache publication.
//! Boundary: adapts native Governance Candid responses into stable public neuron rows.

use super::{
    super::{
        NnsNeuronHostError, enforce_mainnet_network,
        model::{NnsKnownNeuronData, NnsNeuronBallotRow, NnsNeuronRow},
    },
    NnsNeuronPage, NnsNeuronSource, validate_neuron_rows, validate_page_size,
};
use crate::{
    nns::{LiveNnsSource, NnsSourceRequest, governance_query::query_nns_governance},
    runtime::block_on_current_thread,
};
use candid::{CandidType, Deserialize};

const GOVERNANCE_ERROR_TYPE_NOT_FOUND: i32 = 4;

impl NnsNeuronSource for LiveNnsSource {
    fn fetch_neuron_page(
        &self,
        request: &NnsSourceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> Result<NnsNeuronPage, NnsNeuronHostError> {
        enforce_mainnet_network(&request.network)?;
        validate_page_size(page_size)?;
        let wire = block_on_current_thread(fetch_neuron_index_async(
            request,
            exclusive_start_neuron_id,
            page_size,
        ))??;
        let neurons = wire
            .neurons
            .into_iter()
            .map(neuron_row_from_wire)
            .collect::<Result<Vec<_>, _>>()?;
        validate_neuron_rows(&neurons)?;
        let next_start_neuron_id = (neurons.len() == page_size as usize)
            .then(|| neurons.last().map(|row| row.neuron_id))
            .flatten();
        Ok(NnsNeuronPage {
            neurons,
            next_start_neuron_id,
        })
    }

    fn fetch_neuron(
        &self,
        request: &NnsSourceRequest,
        neuron_id: u64,
    ) -> Result<NnsNeuronRow, NnsNeuronHostError> {
        enforce_mainnet_network(&request.network)?;
        let wire = block_on_current_thread(fetch_neuron_info_async(request, neuron_id))??;
        neuron_row_from_wire(wire)
    }
}

async fn fetch_neuron_index_async(
    request: &NnsSourceRequest,
    exclusive_start_neuron_id: Option<u64>,
    page_size: u32,
) -> Result<NeuronIndexDataWire, NnsNeuronHostError> {
    let result: GetNeuronIndexResultWire = query_nns_governance(
        request,
        "get_neuron_index",
        "GetNeuronIndexRequest",
        "GetNeuronIndexResult",
        &GetNeuronIndexRequestWire {
            exclusive_start_neuron_id: exclusive_start_neuron_id.map(|id| NeuronIdWire { id }),
            page_size: Some(page_size),
        },
    )
    .await?;
    governance_result(result)
}

async fn fetch_neuron_info_async(
    request: &NnsSourceRequest,
    neuron_id: u64,
) -> Result<NeuronInfoWire, NnsNeuronHostError> {
    let result: GetNeuronInfoResultWire = query_nns_governance(
        request,
        "get_neuron_info",
        "nat64",
        "NeuronInfoResult",
        &neuron_id,
    )
    .await?;
    governance_result(result).map_err(|error| map_neuron_info_error(error, neuron_id))
}

trait GovernanceResult<Response> {
    fn into_result(self) -> Result<Response, GovernanceErrorWire>;
}

fn governance_result<R, Response>(result: R) -> Result<Response, NnsNeuronHostError>
where
    R: GovernanceResult<Response>,
{
    result
        .into_result()
        .map_err(|error| NnsNeuronHostError::Governance {
            error_type: error.error_type,
            message: error.error_message,
        })
}

fn map_neuron_info_error(error: NnsNeuronHostError, neuron_id: u64) -> NnsNeuronHostError {
    match error {
        NnsNeuronHostError::Governance { error_type, .. }
            if error_type == GOVERNANCE_ERROR_TYPE_NOT_FOUND =>
        {
            NnsNeuronHostError::NeuronNotFound { neuron_id }
        }
        error => error,
    }
}

fn neuron_row_from_wire(wire: NeuronInfoWire) -> Result<NnsNeuronRow, NnsNeuronHostError> {
    let neuron_id = wire.id.ok_or(NnsNeuronHostError::MissingNeuronId)?.id;
    Ok(NnsNeuronRow {
        neuron_id,
        state: wire.state,
        state_text: state_text(wire.state),
        visibility: wire.visibility,
        visibility_text: optional_code_text(wire.visibility, visibility_label),
        neuron_type: wire.neuron_type,
        neuron_type_text: optional_code_text(wire.neuron_type, neuron_type_label),
        stake_e8s: wire.stake_e8s,
        staked_maturity_e8s_equivalent: wire.staked_maturity_e8s_equivalent,
        dissolve_delay_seconds: wire.dissolve_delay_seconds,
        age_seconds: wire.age_seconds,
        created_timestamp_seconds: wire.created_timestamp_seconds,
        retrieved_at_timestamp_seconds: wire.retrieved_at_timestamp_seconds,
        voting_power: wire.voting_power,
        deciding_voting_power: wire.deciding_voting_power,
        potential_voting_power: wire.potential_voting_power,
        voting_power_refreshed_timestamp_seconds: wire.voting_power_refreshed_timestamp_seconds,
        joined_community_fund_timestamp_seconds: wire.joined_community_fund_timestamp_seconds,
        eight_year_gang_bonus_base_e8s: wire.eight_year_gang_bonus_base_e8s,
        known_neuron_data: wire.known_neuron_data.map(|known| NnsKnownNeuronData {
            name: known.name,
            description: known.description,
            links: known.links.unwrap_or_default(),
        }),
        recent_ballots: wire
            .recent_ballots
            .into_iter()
            .map(|ballot| NnsNeuronBallotRow {
                proposal_id: ballot.proposal_id.map(|proposal| proposal.id),
                vote: ballot.vote,
                vote_text: vote_text(ballot.vote),
            })
            .collect(),
    })
}

fn state_text(code: i32) -> String {
    code_text(code, |code| match code {
        0 => Some("unspecified"),
        1 => Some("not-dissolving"),
        2 => Some("dissolving"),
        3 => Some("dissolved"),
        4 => Some("spawning"),
        _ => None,
    })
}

const fn visibility_label(code: i32) -> Option<&'static str> {
    match code {
        0 => Some("unspecified"),
        1 => Some("private"),
        2 => Some("public"),
        _ => None,
    }
}

const fn neuron_type_label(code: i32) -> Option<&'static str> {
    match code {
        0 => Some("unspecified"),
        1 => Some("seed"),
        2 => Some("ect"),
        _ => None,
    }
}

fn vote_text(code: i32) -> String {
    code_text(code, |code| match code {
        0 => Some("unspecified"),
        1 => Some("yes"),
        2 => Some("no"),
        _ => None,
    })
}

fn optional_code_text(
    code: Option<i32>,
    label: impl FnOnce(i32) -> Option<&'static str>,
) -> String {
    code.map_or_else(|| "unknown".to_string(), |code| code_text(code, label))
}

fn code_text(code: i32, label: impl FnOnce(i32) -> Option<&'static str>) -> String {
    label(code).map_or_else(|| format!("unknown({code})"), str::to_string)
}

#[derive(CandidType)]
struct GetNeuronIndexRequestWire {
    exclusive_start_neuron_id: Option<NeuronIdWire>,
    page_size: Option<u32>,
}

#[derive(CandidType, Deserialize)]
enum GetNeuronIndexResultWire {
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

#[derive(CandidType, Deserialize)]
enum GetNeuronInfoResultWire {
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

#[derive(CandidType, Deserialize)]
struct NeuronIndexDataWire {
    neurons: Vec<NeuronInfoWire>,
}

#[derive(CandidType, Deserialize)]
struct GovernanceErrorWire {
    error_message: String,
    error_type: i32,
}

#[derive(CandidType, Deserialize)]
struct NeuronInfoWire {
    id: Option<NeuronIdWire>,
    dissolve_delay_seconds: u64,
    recent_ballots: Vec<BallotInfoWire>,
    neuron_type: Option<i32>,
    created_timestamp_seconds: u64,
    state: i32,
    stake_e8s: u64,
    joined_community_fund_timestamp_seconds: Option<u64>,
    retrieved_at_timestamp_seconds: u64,
    visibility: Option<i32>,
    known_neuron_data: Option<KnownNeuronDataWire>,
    age_seconds: u64,
    voting_power: u64,
    voting_power_refreshed_timestamp_seconds: Option<u64>,
    deciding_voting_power: Option<u64>,
    potential_voting_power: Option<u64>,
    eight_year_gang_bonus_base_e8s: Option<u64>,
    staked_maturity_e8s_equivalent: Option<u64>,
}

#[derive(CandidType, Deserialize)]
struct NeuronIdWire {
    id: u64,
}

#[derive(CandidType, Deserialize)]
struct ProposalIdWire {
    id: u64,
}

#[derive(CandidType, Deserialize)]
struct BallotInfoWire {
    vote: i32,
    proposal_id: Option<ProposalIdWire>,
}

#[derive(CandidType, Deserialize)]
struct KnownNeuronDataWire {
    name: String,
    description: Option<String>,
    links: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::{
        NnsNeuronHostError, map_neuron_info_error, neuron_type_label, state_text, visibility_label,
        vote_text,
    };

    #[test]
    fn governance_discriminants_keep_stable_native_labels() {
        assert_eq!(state_text(0), "unspecified");
        assert_eq!(state_text(1), "not-dissolving");
        assert_eq!(state_text(2), "dissolving");
        assert_eq!(state_text(3), "dissolved");
        assert_eq!(state_text(4), "spawning");
        assert_eq!(state_text(99), "unknown(99)");

        assert_eq!(visibility_label(0), Some("unspecified"));
        assert_eq!(visibility_label(1), Some("private"));
        assert_eq!(visibility_label(2), Some("public"));
        assert_eq!(visibility_label(99), None);

        assert_eq!(neuron_type_label(0), Some("unspecified"));
        assert_eq!(neuron_type_label(1), Some("seed"));
        assert_eq!(neuron_type_label(2), Some("ect"));
        assert_eq!(neuron_type_label(99), None);

        assert_eq!(vote_text(0), "unspecified");
        assert_eq!(vote_text(1), "yes");
        assert_eq!(vote_text(2), "no");
        assert_eq!(vote_text(99), "unknown(99)");
    }

    #[test]
    fn neuron_info_maps_only_the_native_not_found_error() {
        let not_found = map_neuron_info_error(
            NnsNeuronHostError::Governance {
                error_type: 4,
                message: "wording is not part of the contract".to_string(),
            },
            42,
        );
        assert!(matches!(
            not_found,
            NnsNeuronHostError::NeuronNotFound { neuron_id: 42 }
        ));

        let unrelated = map_neuron_info_error(
            NnsNeuronHostError::Governance {
                error_type: 12,
                message: "not found text must not override the code".to_string(),
            },
            42,
        );
        assert!(matches!(
            unrelated,
            NnsNeuronHostError::Governance { error_type: 12, .. }
        ));
    }
}
