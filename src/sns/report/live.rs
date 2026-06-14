use super::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsCanisters, MainnetSnsList,
    MainnetSnsNeuronPage, MainnetSnsNeurons, MainnetSnsProposal, MainnetSnsProposals,
    MainnetSnsToken, SNS_METADATA_CONCURRENCY, SNS_TOKEN_LOGO_METADATA_KEY, SnsFetchRequest,
    SnsGovernanceParameters, SnsHostError, SnsListSource, SnsNeuronId, SnsNeuronRow,
    SnsNeuronsSource, SnsParamsSource, SnsProposalBallotRow, SnsProposalFailureReason,
    SnsProposalRow, SnsProposalSource, SnsProposalTally, SnsProposalsSource, SnsTokenMetadataRow,
    SnsTokenSource, SnsTokenStandardRow, hex_bytes, short_principal,
};
use crate::{
    runtime::block_on_current_thread,
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use candid::{CandidType, Decode, Deserialize, Encode, Int, Nat, Principal};
use futures::{StreamExt, stream};
use ic_agent::Agent;
use serde_json::Value as JsonValue;

pub(super) struct LiveSnsListSource;

impl SnsListSource for LiveSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        fetch_mainnet_sns_list(request)
    }
}

impl SnsTokenSource for LiveSnsListSource {
    fn fetch_sns_token(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError> {
        fetch_mainnet_sns_token(request, sns)
    }
}

impl SnsParamsSource for LiveSnsListSource {
    fn fetch_sns_params(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        fetch_mainnet_sns_params(request, sns)
    }
}

impl SnsProposalSource for LiveSnsListSource {
    fn fetch_sns_proposal(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError> {
        fetch_mainnet_sns_proposal(request, sns, proposal_id)
    }
}

impl SnsProposalsSource for LiveSnsListSource {
    fn fetch_sns_proposals(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
    ) -> Result<MainnetSnsProposals, SnsHostError> {
        fetch_mainnet_sns_proposals(request, sns, limit, before_proposal_id, include_status)
    }
}

impl SnsNeuronsSource for LiveSnsListSource {
    fn fetch_sns_neurons(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        fetch_mainnet_sns_neurons(request, sns, limit, owner_principal_id)
    }

    fn fetch_sns_neuron_page(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        fetch_mainnet_sns_neuron_page(request, sns, limit, start_page_at, owner_principal_id)
    }
}

fn fetch_mainnet_sns_list(request: &SnsFetchRequest) -> Result<MainnetSnsList, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_list_async(request)).map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_token(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsToken, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_token_async(request, sns))
        .map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_params(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_params_async(request, sns))
        .map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_proposal(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    proposal_id: u64,
) -> Result<MainnetSnsProposal, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_proposal_async(request, sns, proposal_id))
        .map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_proposals(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
) -> Result<MainnetSnsProposals, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_proposals_async(
        request,
        sns,
        limit,
        before_proposal_id,
        include_status,
    ))
    .map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_neurons(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeurons, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_neurons_async(
        request,
        sns,
        limit,
        owner_principal_id,
    ))
    .map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_neuron_page(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeuronPage, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_neuron_page_async(
        request,
        sns,
        limit,
        start_page_at,
        owner_principal_id,
    ))
    .map_err(SnsHostError::Runtime)?
}

async fn fetch_mainnet_sns_list_async(
    request: &SnsFetchRequest,
) -> Result<MainnetSnsList, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let sns_wasm_canister =
        principal_from_text(MAINNET_SNS_WASM_CANISTER_ID, "sns_wasm_canister_id")?;
    let arg = Encode!(&ListDeployedSnsesRequest {}).map_err(|err| SnsHostError::CandidEncode {
        message: "ListDeployedSnsesRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&sns_wasm_canister, "list_deployed_snses")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "list_deployed_snses",
            reason: err.to_string(),
        })?;
    let response =
        Decode!(&bytes, ListDeployedSnsesResponse).map_err(|err| SnsHostError::CandidDecode {
            message: "ListDeployedSnsesResponse",
            reason: err.to_string(),
        })?;
    mainnet_sns_list_from_response(&agent, request, response).await
}

async fn fetch_mainnet_sns_token_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsToken, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let ledger_canister = principal_from_text(&sns.ledger_canister_id, "ledger_canister_id")?;
    let token_name = query_ledger(&agent, &ledger_canister, "icrc1_name").await?;
    let token_symbol = query_ledger(&agent, &ledger_canister, "icrc1_symbol").await?;
    let decimals = query_ledger(&agent, &ledger_canister, "icrc1_decimals").await?;
    let transfer_fee: Nat = query_ledger(&agent, &ledger_canister, "icrc1_fee").await?;
    let total_supply: Nat = query_ledger(&agent, &ledger_canister, "icrc1_total_supply").await?;
    let minting_account: Option<IcrcAccount> =
        query_ledger(&agent, &ledger_canister, "icrc1_minting_account").await?;
    let supported_standards: Vec<IcrcSupportedStandard> =
        query_ledger(&agent, &ledger_canister, "icrc1_supported_standards").await?;
    let metadata: Vec<(String, IcrcMetadataValue)> =
        query_ledger(&agent, &ledger_canister, "icrc1_metadata").await?;
    let (ledger_index_canister_id, ledger_index_error) =
        match query_ledger::<GetIndexPrincipalResult>(
            &agent,
            &ledger_canister,
            "icrc106_get_index_principal",
        )
        .await
        {
            Ok(GetIndexPrincipalResult::Ok(principal)) => (Some(principal.to_text()), None),
            Ok(GetIndexPrincipalResult::Err(error)) => {
                (None, Some(index_principal_error_text(error)))
            }
            Err(error) => (None, Some(error.to_string())),
        };

    Ok(MainnetSnsToken {
        token_name,
        token_symbol,
        decimals,
        transfer_fee: transfer_fee.to_string(),
        total_supply: total_supply.to_string(),
        minting_account_owner: minting_account
            .as_ref()
            .map(|account| account.owner.to_text()),
        minting_account_subaccount_hex: minting_account
            .as_ref()
            .and_then(|account| account.subaccount.as_deref())
            .map(hex_bytes),
        ledger_index_canister_id,
        ledger_index_error,
        supported_standards: supported_standards
            .into_iter()
            .map(|standard| SnsTokenStandardRow {
                name: standard.name,
                url: standard.url,
            })
            .collect(),
        metadata: metadata
            .into_iter()
            .map(|(key, value)| metadata_row(key, value))
            .collect(),
    })
}

async fn fetch_mainnet_sns_params_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let arg = Encode!(&()).map_err(|err| SnsHostError::CandidEncode {
        message: "get_nervous_system_parameters",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&governance_canister, "get_nervous_system_parameters")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "get_nervous_system_parameters",
            reason: err.to_string(),
        })?;
    Decode!(&bytes, SnsGovernanceParameters).map_err(|err| SnsHostError::CandidDecode {
        message: "SnsGovernanceParameters",
        reason: err.to_string(),
    })
}

async fn fetch_mainnet_sns_proposal_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    proposal_id: u64,
) -> Result<MainnetSnsProposal, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let arg = Encode!(&GetProposalRequest {
        proposal_id: Some(SnsProposalId { id: proposal_id }),
    })
    .map_err(|err| SnsHostError::CandidEncode {
        message: "GetProposalRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&governance_canister, "get_proposal")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "get_proposal",
            reason: err.to_string(),
        })?;
    let response =
        Decode!(&bytes, GetProposalResponse).map_err(|err| SnsHostError::CandidDecode {
            message: "GetProposalResponse",
            reason: err.to_string(),
        })?;
    match response.result {
        Some(GetProposalResult::Proposal(proposal)) => Ok(MainnetSnsProposal {
            proposal: sns_proposal_row(*proposal),
        }),
        Some(GetProposalResult::Error(err)) => Err(SnsHostError::GovernanceError {
            method: "get_proposal",
            error_type: err.error_type,
            message: err.error_message,
        }),
        None => Err(SnsHostError::MissingGovernanceResult {
            method: "get_proposal",
        }),
    }
}

async fn fetch_mainnet_sns_proposals_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
) -> Result<MainnetSnsProposals, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let arg = Encode!(&ListProposalsRequest {
        include_reward_status: Vec::new(),
        before_proposal: before_proposal_id.map(|id| SnsProposalId { id }),
        limit,
        exclude_type: Vec::new(),
        include_status: include_status.to_vec(),
        include_topics: None,
    })
    .map_err(|err| SnsHostError::CandidEncode {
        message: "ListProposalsRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&governance_canister, "list_proposals")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "list_proposals",
            reason: err.to_string(),
        })?;
    let response =
        Decode!(&bytes, ListProposalsResponse).map_err(|err| SnsHostError::CandidDecode {
            message: "ListProposalsResponse",
            reason: err.to_string(),
        })?;
    Ok(MainnetSnsProposals {
        proposals: response
            .proposals
            .into_iter()
            .map(sns_proposal_row)
            .collect(),
    })
}

async fn fetch_mainnet_sns_neurons_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeurons, SnsHostError> {
    let page =
        fetch_mainnet_sns_neuron_page_async(request, sns, limit, None, owner_principal_id).await?;
    Ok(MainnetSnsNeurons {
        neurons: page.neurons,
    })
}

async fn fetch_mainnet_sns_neuron_page_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeuronPage, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let owner_principal = owner_principal_id
        .map(|principal| principal_from_text(principal, "owner_principal_id"))
        .transpose()?;
    let arg = Encode!(&ListNeuronsRequest {
        of_principal: owner_principal,
        limit,
        start_page_at: start_page_at.cloned(),
    })
    .map_err(|err| SnsHostError::CandidEncode {
        message: "ListNeuronsRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&governance_canister, "list_neurons")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "list_neurons",
            reason: err.to_string(),
        })?;
    let response =
        Decode!(&bytes, ListNeuronsResponse).map_err(|err| SnsHostError::CandidDecode {
            message: "ListNeuronsResponse",
            reason: err.to_string(),
        })?;
    let last_cursor = response.neurons.iter().rev().find_map(sns_neuron_cursor);
    Ok(MainnetSnsNeuronPage {
        neurons: response.neurons.into_iter().map(sns_neuron_row).collect(),
        last_cursor,
    })
}

async fn query_ledger<T>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
) -> Result<T, SnsHostError>
where
    T: for<'de> Deserialize<'de> + CandidType,
{
    let arg = Encode!().map_err(|err| SnsHostError::CandidEncode {
        message: method,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(ledger_canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| SnsHostError::CandidDecode {
        message: method,
        reason: err.to_string(),
    })
}

fn sns_agent(endpoint: &str) -> Result<Agent, SnsHostError> {
    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|err| SnsHostError::AgentBuild {
            endpoint: endpoint.to_string(),
            reason: err.to_string(),
        })
}

async fn mainnet_sns_list_from_response(
    agent: &Agent,
    request: &SnsFetchRequest,
    response: ListDeployedSnsesResponse,
) -> Result<MainnetSnsList, SnsHostError> {
    let sns_canisters = response
        .instances
        .into_iter()
        .map(mainnet_sns_canisters_from_deployed_sns)
        .collect::<Result<Vec<_>, _>>()?;
    let fetched = stream::iter(
        sns_canisters
            .into_iter()
            .map(|sns| fetch_mainnet_sns_metadata(agent, sns)),
    )
    .buffered(SNS_METADATA_CONCURRENCY)
    .collect::<Vec<_>>()
    .await;
    let mut sns_instances = Vec::with_capacity(fetched.len());
    for sns in fetched {
        sns_instances.push(sns?);
    }
    Ok(MainnetSnsList {
        network: MAINNET_NETWORK.to_string(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        sns_instances,
    })
}

async fn fetch_mainnet_sns_metadata(
    agent: &Agent,
    sns: MainnetSnsCanisters,
) -> Result<MainnetSns, SnsHostError> {
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let (metadata, metadata_error) =
        match fetch_governance_metadata(agent, &governance_canister).await {
            Ok(metadata) => (metadata, None),
            Err(err) => match metadata_error_summary(&err) {
                Some(summary) => (GetMetadataResponse::default(), Some(summary)),
                None => return Err(err),
            },
        };
    Ok(mainnet_sns_from_canisters_and_metadata(
        sns,
        metadata,
        metadata_error,
    ))
}

async fn fetch_governance_metadata(
    agent: &Agent,
    governance_canister: &Principal,
) -> Result<GetMetadataResponse, SnsHostError> {
    let arg = Encode!(&GetMetadataRequest {}).map_err(|err| SnsHostError::CandidEncode {
        message: "GetMetadataRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(governance_canister, "get_metadata")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "get_metadata",
            reason: err.to_string(),
        })?;
    let metadata =
        Decode!(&bytes, GetMetadataResponse).map_err(|err| SnsHostError::CandidDecode {
            message: "GetMetadataResponse",
            reason: err.to_string(),
        })?;
    Ok(metadata)
}

fn mainnet_sns_canisters_from_deployed_sns(
    sns: DeployedSns,
) -> Result<MainnetSnsCanisters, SnsHostError> {
    Ok(MainnetSnsCanisters {
        root_canister_id: required_principal_text(sns.root_canister_id, "root_canister_id")?,
        governance_canister_id: required_principal_text(
            sns.governance_canister_id,
            "governance_canister_id",
        )?,
        ledger_canister_id: required_principal_text(sns.ledger_canister_id, "ledger_canister_id")?,
        swap_canister_id: required_principal_text(sns.swap_canister_id, "swap_canister_id")?,
        index_canister_id: required_principal_text(sns.index_canister_id, "index_canister_id")?,
    })
}

fn mainnet_sns_from_canisters_and_metadata(
    sns: MainnetSnsCanisters,
    metadata: GetMetadataResponse,
    metadata_error: Option<String>,
) -> MainnetSns {
    let name = clean_optional_text(metadata.name)
        .unwrap_or_else(|| format!("unnamed-{}", short_principal(&sns.root_canister_id)));
    MainnetSns {
        id: 0,
        name,
        description: clean_optional_text(metadata.description),
        url: clean_optional_text(metadata.url),
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        swap_canister_id: sns.swap_canister_id,
        index_canister_id: sns.index_canister_id,
        metadata_error,
    }
}

fn required_principal_text(
    principal: Option<Principal>,
    field: &'static str,
) -> Result<String, SnsHostError> {
    principal
        .map(|principal| principal.to_text())
        .ok_or_else(|| SnsHostError::InvalidPrincipal {
            field,
            reason: "missing principal".to_string(),
        })
}

fn principal_from_text(value: &str, field: &'static str) -> Result<Principal, SnsHostError> {
    Principal::from_text(value).map_err(|err| SnsHostError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}

fn clean_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn sns_neuron_row(neuron: SnsGovernanceNeuron) -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: neuron
            .id
            .map_or_else(|| "-".to_string(), |id| hex_bytes(&id.id)),
        cached_neuron_stake_e8s: neuron.cached_neuron_stake_e8s,
        maturity_e8s_equivalent: neuron.maturity_e8s_equivalent,
        staked_maturity_e8s_equivalent: neuron.staked_maturity_e8s_equivalent,
        created_timestamp_seconds: neuron.created_timestamp_seconds,
        created_at: format_utc_timestamp_secs(neuron.created_timestamp_seconds),
    }
}

fn sns_neuron_cursor(neuron: &SnsGovernanceNeuron) -> Option<SnsNeuronId> {
    neuron.id.clone()
}

fn sns_proposal_row(proposal: SnsGovernanceProposalData) -> SnsProposalRow {
    let decision_state = proposal_decision_state(&proposal);
    let proposal_id = proposal.id.as_ref().map(|id| id.id);
    let proposal_fields = proposal.proposal.unwrap_or_default();
    let ballots = proposal
        .ballots
        .into_iter()
        .map(sns_proposal_ballot_row)
        .collect::<Vec<_>>();
    let ballot_count = ballots.len();
    SnsProposalRow {
        proposal_id,
        action_id: proposal.action,
        action: proposal_action_text(proposal.action),
        title: proposal_fields.title,
        summary: proposal_fields.summary,
        url: clean_optional_text(Some(proposal_fields.url)),
        decision_state,
        reject_cost_e8s: proposal.reject_cost_e8s,
        proposal_creation_timestamp_seconds: proposal.proposal_creation_timestamp_seconds,
        created_at: format_utc_timestamp_secs(proposal.proposal_creation_timestamp_seconds),
        decided_timestamp_seconds: nonzero_timestamp(proposal.decided_timestamp_seconds),
        decided_at: optional_timestamp_text(proposal.decided_timestamp_seconds),
        executed_timestamp_seconds: nonzero_timestamp(proposal.executed_timestamp_seconds),
        executed_at: optional_timestamp_text(proposal.executed_timestamp_seconds),
        failed_timestamp_seconds: nonzero_timestamp(proposal.failed_timestamp_seconds),
        failed_at: optional_timestamp_text(proposal.failed_timestamp_seconds),
        failure_reason: proposal
            .failure_reason
            .map(|reason| SnsProposalFailureReason {
                error_type: reason.error_type,
                error_message: reason.error_message,
            }),
        reward_event_round: proposal.reward_event_round,
        reward_event_end_timestamp_seconds: proposal.reward_event_end_timestamp_seconds,
        is_eligible_for_rewards: proposal.is_eligible_for_rewards,
        latest_tally: proposal.latest_tally.map(|tally| SnsProposalTally {
            timestamp_seconds: tally.timestamp_seconds,
            yes: tally.yes,
            no: tally.no,
            total: tally.total,
        }),
        ballot_count,
        ballots,
        payload_text_rendering: proposal
            .payload_text_rendering
            .and_then(|value| clean_optional_text(Some(value))),
        proposer_neuron_id: proposal.proposer.map(|id| hex_bytes(&id.id)),
    }
}

fn sns_proposal_ballot_row(
    (neuron_id, ballot): (String, SnsGovernanceBallot),
) -> SnsProposalBallotRow {
    SnsProposalBallotRow {
        neuron_id,
        vote: ballot.vote,
        vote_text: ballot_vote_text(ballot.vote),
        cast_timestamp_seconds: ballot.cast_timestamp_seconds,
        cast_at: optional_timestamp_text(ballot.cast_timestamp_seconds),
        voting_power: ballot.voting_power,
    }
}

fn ballot_vote_text(vote: i32) -> String {
    match vote {
        0 => "unspecified".to_string(),
        1 => "yes".to_string(),
        2 => "no".to_string(),
        other => format!("unknown:{other}"),
    }
}

fn proposal_decision_state(proposal: &SnsGovernanceProposalData) -> String {
    if proposal.failed_timestamp_seconds > 0 {
        "failed"
    } else if proposal.executed_timestamp_seconds > 0 {
        "executed"
    } else if proposal.decided_timestamp_seconds > 0 {
        "decided"
    } else {
        "open"
    }
    .to_string()
}

fn proposal_action_text(action: u64) -> String {
    match action {
        0 => "unspecified".to_string(),
        1 => "motion".to_string(),
        2 => "manage_nervous_system_parameters".to_string(),
        3 => "upgrade_sns_controlled_canister".to_string(),
        4 => "add_generic_nervous_system_function".to_string(),
        5 => "remove_generic_nervous_system_function".to_string(),
        6 => "execute_generic_nervous_system_function".to_string(),
        7 => "upgrade_sns_to_next_version".to_string(),
        8 => "manage_sns_metadata".to_string(),
        9 => "transfer_sns_treasury_funds".to_string(),
        10 => "register_dapp_canisters".to_string(),
        11 => "deregister_dapp_canisters".to_string(),
        12 => "mint_sns_tokens".to_string(),
        13 => "manage_ledger_parameters".to_string(),
        14 => "manage_dapp_canister_settings".to_string(),
        15 => "advance_sns_target_version".to_string(),
        16 => "set_topics_for_custom_proposals".to_string(),
        17 => "register_extension".to_string(),
        18 => "execute_extension_operation".to_string(),
        19 => "upgrade_extension".to_string(),
        id if id >= 1_000 => format!("generic:{id}"),
        id => format!("unknown:{id}"),
    }
}

fn nonzero_timestamp(timestamp_seconds: u64) -> Option<u64> {
    (timestamp_seconds > 0).then_some(timestamp_seconds)
}

fn optional_timestamp_text(timestamp_seconds: u64) -> Option<String> {
    nonzero_timestamp(timestamp_seconds).map(format_utc_timestamp_secs)
}

pub(super) fn metadata_row(key: String, value: IcrcMetadataValue) -> SnsTokenMetadataRow {
    if key == SNS_TOKEN_LOGO_METADATA_KEY {
        return SnsTokenMetadataRow {
            key,
            value_type: "bool".to_string(),
            value: JsonValue::Bool(metadata_value_is_present(&value)),
        };
    }

    let (value_type, value) = match value {
        IcrcMetadataValue::Nat(value) => ("nat", value.to_string()),
        IcrcMetadataValue::Int(value) => ("int", value.to_string()),
        IcrcMetadataValue::Text(value) => ("text", value),
        IcrcMetadataValue::Blob(value) => ("blob", hex_bytes(&value)),
    };
    SnsTokenMetadataRow {
        key,
        value_type: value_type.to_string(),
        value: JsonValue::String(value),
    }
}

fn metadata_value_is_present(value: &IcrcMetadataValue) -> bool {
    match value {
        IcrcMetadataValue::Text(value) => !value.trim().is_empty(),
        IcrcMetadataValue::Blob(value) => !value.is_empty(),
        IcrcMetadataValue::Nat(_) | IcrcMetadataValue::Int(_) => true,
    }
}

fn index_principal_error_text(error: GetIndexPrincipalError) -> String {
    match error {
        GetIndexPrincipalError::IndexPrincipalNotSet => "index principal not set".to_string(),
        GetIndexPrincipalError::GenericError {
            error_code,
            description,
        } => format!("generic error {error_code}: {description}"),
    }
}

fn metadata_error_summary(err: &SnsHostError) -> Option<String> {
    match err {
        SnsHostError::AgentCall { method, reason } => Some(format!("{method}: {reason}")),
        SnsHostError::CandidEncode { message, reason } => {
            Some(format!("encode {message}: {reason}"))
        }
        SnsHostError::CandidDecode { message, reason } => {
            Some(format!("decode {message}: {reason}"))
        }
        SnsHostError::GovernanceError {
            method,
            error_type,
            message,
        } => Some(format!("{method} governance error {error_type}: {message}")),
        SnsHostError::MissingGovernanceResult { method } => {
            Some(format!("{method}: missing governance result"))
        }
        SnsHostError::UnsupportedNetwork { .. }
        | SnsHostError::Runtime(_)
        | SnsHostError::AgentBuild { .. }
        | SnsHostError::InvalidPrincipal { .. }
        | SnsHostError::UnknownSnsId { .. }
        | SnsHostError::UnknownSnsRoot { .. }
        | SnsHostError::InvalidLookup { .. }
        | SnsHostError::MissingNeuronsCache { .. }
        | SnsHostError::MissingNeuronsCacheForId { .. }
        | SnsHostError::ReadCache { .. }
        | SnsHostError::ParseCache { .. }
        | SnsHostError::SerializeCache { .. }
        | SnsHostError::UnsupportedCacheSchemaVersion { .. }
        | SnsHostError::CacheNetworkMismatch { .. }
        | SnsHostError::Cache(_)
        | SnsHostError::IncompleteRefresh { .. }
        | SnsHostError::MissingCacheRoot => None,
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListDeployedSnsesRequest {}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListDeployedSnsesResponse {
    instances: Vec<DeployedSns>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct DeployedSns {
    root_canister_id: Option<Principal>,
    governance_canister_id: Option<Principal>,
    ledger_canister_id: Option<Principal>,
    swap_canister_id: Option<Principal>,
    index_canister_id: Option<Principal>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct GetMetadataRequest {}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
struct GetMetadataResponse {
    url: Option<String>,
    logo: Option<String>,
    name: Option<String>,
    description: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcAccount {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) enum IcrcMetadataValue {
    Nat(Nat),
    Int(Int),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcSupportedStandard {
    name: String,
    url: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum GetIndexPrincipalResult {
    Ok(Principal),
    Err(GetIndexPrincipalError),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum GetIndexPrincipalError {
    IndexPrincipalNotSet,
    GenericError {
        error_code: Nat,
        description: String,
    },
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListNeuronsRequest {
    of_principal: Option<Principal>,
    limit: u32,
    start_page_at: Option<SnsNeuronId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListNeuronsResponse {
    neurons: Vec<SnsGovernanceNeuron>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct GetProposalRequest {
    proposal_id: Option<SnsProposalId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct GetProposalResponse {
    result: Option<GetProposalResult>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum GetProposalResult {
    Error(SnsGovernanceError),
    Proposal(Box<SnsGovernanceProposalData>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListProposalsRequest {
    include_reward_status: Vec<i32>,
    before_proposal: Option<SnsProposalId>,
    limit: u32,
    exclude_type: Vec<u64>,
    include_status: Vec<i32>,
    include_topics: Option<Vec<SnsTopicSelector>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsTopicSelector {
    topic: Option<SnsTopic>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum SnsTopic {
    DaoCommunitySettings,
    SnsFrameworkManagement,
    DappCanisterManagement,
    ApplicationBusinessLogic,
    Governance,
    TreasuryAssetManagement,
    CriticalDappOperations,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListProposalsResponse {
    proposals: Vec<SnsGovernanceProposalData>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsProposalId {
    id: u64,
}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
struct SnsGovernanceProposal {
    title: String,
    summary: String,
    url: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsGovernanceError {
    error_type: i32,
    error_message: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsGovernanceBallot {
    vote: i32,
    cast_timestamp_seconds: u64,
    voting_power: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsGovernanceProposalTally {
    timestamp_seconds: u64,
    yes: u64,
    no: u64,
    total: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsGovernanceProposalData {
    id: Option<SnsProposalId>,
    payload_text_rendering: Option<String>,
    action: u64,
    failure_reason: Option<SnsGovernanceError>,
    ballots: Vec<(String, SnsGovernanceBallot)>,
    reward_event_round: u64,
    failed_timestamp_seconds: u64,
    reward_event_end_timestamp_seconds: Option<u64>,
    proposal_creation_timestamp_seconds: u64,
    reject_cost_e8s: u64,
    latest_tally: Option<SnsGovernanceProposalTally>,
    decided_timestamp_seconds: u64,
    proposal: Option<SnsGovernanceProposal>,
    proposer: Option<SnsNeuronId>,
    is_eligible_for_rewards: bool,
    executed_timestamp_seconds: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsGovernanceNeuron {
    id: Option<SnsNeuronId>,
    staked_maturity_e8s_equivalent: Option<u64>,
    maturity_e8s_equivalent: u64,
    cached_neuron_stake_e8s: u64,
    created_timestamp_seconds: u64,
}
