mod convert;
mod types;

use super::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsCanisters, MainnetSnsList,
    MainnetSnsNeuronPage, MainnetSnsNeurons, MainnetSnsProposal, MainnetSnsProposals,
    MainnetSnsToken, SNS_METADATA_CONCURRENCY, SnsFetchRequest, SnsGovernanceParameters,
    SnsHostError, SnsListSource, SnsNeuronId, SnsNeuronsSource, SnsParamsSource, SnsProposalSource,
    SnsProposalsSource, SnsTokenSource, SnsTokenStandardRow, hex_bytes,
};
use crate::{runtime::block_on_current_thread, subnet_catalog::MAINNET_NETWORK};
use candid::{CandidType, Deserialize, Encode, Nat, Principal};
use futures::{StreamExt, stream};
use ic_agent::Agent;

pub(super) use convert::metadata_row;
use convert::{
    index_principal_error_text, mainnet_sns_canisters_from_deployed_sns,
    mainnet_sns_from_canisters_and_metadata, metadata_error_summary, sns_neuron_cursor,
    sns_neuron_row, sns_proposal_row,
};
pub(super) use types::IcrcMetadataValue;
use types::*;

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
    let response: ListDeployedSnsesResponse = query_canister(
        &agent,
        &sns_wasm_canister,
        "list_deployed_snses",
        "ListDeployedSnsesRequest",
        "ListDeployedSnsesResponse",
        &ListDeployedSnsesRequest {},
    )
    .await?;
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
    query_canister(
        &agent,
        &governance_canister,
        "get_nervous_system_parameters",
        "get_nervous_system_parameters",
        "SnsGovernanceParameters",
        &(),
    )
    .await
}

async fn fetch_mainnet_sns_proposal_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    proposal_id: u64,
) -> Result<MainnetSnsProposal, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let response: GetProposalResponse = query_canister(
        &agent,
        &governance_canister,
        "get_proposal",
        "GetProposalRequest",
        "GetProposalResponse",
        &GetProposalRequest {
            proposal_id: Some(SnsProposalId { id: proposal_id }),
        },
    )
    .await?;
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
    let response: ListProposalsResponse = query_canister(
        &agent,
        &governance_canister,
        "list_proposals",
        "ListProposalsRequest",
        "ListProposalsResponse",
        &ListProposalsRequest {
            include_reward_status: Vec::new(),
            before_proposal: before_proposal_id.map(|id| SnsProposalId { id }),
            limit,
            exclude_type: Vec::new(),
            include_status: include_status.to_vec(),
            include_topics: None,
        },
    )
    .await?;
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
    let response: ListNeuronsResponse = query_canister(
        &agent,
        &governance_canister,
        "list_neurons",
        "ListNeuronsRequest",
        "ListNeuronsResponse",
        &ListNeuronsRequest {
            of_principal: owner_principal,
            limit,
            start_page_at: start_page_at.cloned(),
        },
    )
    .await?;
    let last_cursor = response.neurons.iter().rev().find_map(sns_neuron_cursor);
    Ok(MainnetSnsNeuronPage {
        neurons: response.neurons.into_iter().map(sns_neuron_row).collect(),
        last_cursor,
    })
}

async fn query_canister<Arg, Response>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, SnsHostError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|err| SnsHostError::CandidEncode {
        message: request_message,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| SnsHostError::CandidDecode {
        message: response_message,
        reason: err.to_string(),
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
    query_canister(
        agent,
        governance_canister,
        "get_metadata",
        "GetMetadataRequest",
        "GetMetadataResponse",
        &GetMetadataRequest {},
    )
    .await
}

fn principal_from_text(value: &str, field: &'static str) -> Result<Principal, SnsHostError> {
    Principal::from_text(value).map_err(|err| SnsHostError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}
