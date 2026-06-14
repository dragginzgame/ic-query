use crate::{
    ic_registry::DEFAULT_MAINNET_ENDPOINT,
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use candid::Principal;
use live::LiveSnsListSource;
pub use model::*;
use source::{
    MainnetSns, MainnetSnsCanisters, MainnetSnsList, MainnetSnsNeuronPage, MainnetSnsNeurons,
    MainnetSnsProposal, MainnetSnsProposals, MainnetSnsToken, SnsFetchRequest, SnsListSource,
    SnsNeuronId, SnsNeuronsSource, SnsParamsSource, SnsProposalSource, SnsProposalsSource,
    SnsTokenSource,
};

mod live;
mod model;
mod neurons_cache;
mod source;
mod text;

#[cfg(test)]
use neurons_cache::{
    SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, SNS_NEURONS_CACHE_SCHEMA_VERSION,
    SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION, refresh_sns_neurons_cache_with_source,
    sns_neurons_cache_path, sns_neurons_refresh_attempt_path, sns_neurons_refresh_lock_path,
};
pub use neurons_cache::{
    build_sns_neurons_cache_list_report, build_sns_neurons_cache_status_report,
    refresh_sns_neurons_cache,
};
pub use text::{
    sns_info_report_text, sns_list_report_text, sns_neurons_cache_list_report_text,
    sns_neurons_cache_status_report_text, sns_neurons_refresh_report_text, sns_neurons_report_text,
    sns_params_report_text, sns_proposal_report_text, sns_proposals_report_text,
    sns_token_report_text,
};

#[cfg(test)]
use live::{IcrcMetadataValue, metadata_row};

pub const DEFAULT_SNS_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const MAINNET_SNS_WASM_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";

const SNS_LIST_REPORT_SCHEMA_VERSION: u32 = 3;
const SNS_INFO_REPORT_SCHEMA_VERSION: u32 = 2;
const SNS_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_PARAMS_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_PROPOSAL_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_PROPOSALS_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_NEURONS_REPORT_SCHEMA_VERSION: u32 = 1;
const COMPACT_PRINCIPAL_CHARS: usize = 5;
const SNS_TOKEN_LOGO_METADATA_KEY: &str = "icrc1:logo";
const SNS_METADATA_CONCURRENCY: usize = 16;

pub fn build_sns_list_report(request: &SnsListRequest) -> Result<SnsListReport, SnsHostError> {
    build_sns_list_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_info_report(request: &SnsInfoRequest) -> Result<SnsInfoReport, SnsHostError> {
    build_sns_info_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_params_report(
    request: &SnsParamsRequest,
) -> Result<SnsParamsReport, SnsHostError> {
    build_sns_params_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_token_report(request: &SnsTokenRequest) -> Result<SnsTokenReport, SnsHostError> {
    build_sns_token_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_proposal_report(
    request: &SnsProposalRequest,
) -> Result<SnsProposalReport, SnsHostError> {
    build_sns_proposal_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_proposals_report(
    request: &SnsProposalsRequest,
) -> Result<SnsProposalsReport, SnsHostError> {
    build_sns_proposals_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_neurons_report(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    build_sns_neurons_report_with_source(request, &LiveSnsListSource)
}

fn build_sns_list_report_with_source(
    request: &SnsListRequest,
    source: &dyn SnsListSource,
) -> Result<SnsListReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetch_request = fetch_request_from_parts(
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    );
    let mut list = source.fetch_deployed_snses(&fetch_request)?;
    assign_sns_ids_in_current_order(&mut list.sns_instances);
    sort_mainnet_sns_instances(&mut list.sns_instances, request.sort);
    Ok(sns_list_report_from_list(
        list,
        request.verbose,
        request.sort,
    ))
}

fn build_sns_info_report_with_source(
    request: &SnsInfoRequest,
    source: &dyn SnsListSource,
) -> Result<SnsInfoReport, SnsHostError> {
    let (_fetch_request, list, id, sns) = resolve_sns_lookup(request, source)?;
    Ok(sns_info_report_from_list(list, id, sns))
}

fn build_sns_params_report_with_source(
    request: &SnsParamsRequest,
    source: &dyn SnsParamsSource,
) -> Result<SnsParamsReport, SnsHostError> {
    let (fetch_request, list, id, sns) = resolve_sns_lookup(request, source)?;
    let parameters = source.fetch_sns_params(&fetch_request, &sns)?;
    Ok(sns_params_report_from_parts(list, id, sns, parameters))
}

fn build_sns_token_report_with_source(
    request: &SnsTokenRequest,
    source: &dyn SnsTokenSource,
) -> Result<SnsTokenReport, SnsHostError> {
    let (fetch_request, list, id, sns) = resolve_sns_lookup(request, source)?;
    let token = source.fetch_sns_token(&fetch_request, &sns)?;
    Ok(sns_token_report_from_parts(list, id, sns, token))
}

fn build_sns_proposal_report_with_source(
    request: &SnsProposalRequest,
    source: &dyn SnsProposalSource,
) -> Result<SnsProposalReport, SnsHostError> {
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let (fetch_request, list, id, sns) = resolve_sns_lookup(&lookup_request, source)?;
    let proposal = source.fetch_sns_proposal(&fetch_request, &sns, request.proposal_id)?;
    Ok(sns_proposal_report_from_parts(SnsProposalReportParts {
        list,
        id,
        sns,
        proposal_id: request.proposal_id,
        verbose: request.verbose,
        proposal,
    }))
}

fn build_sns_proposals_report_with_source(
    request: &SnsProposalsRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let (fetch_request, list, id, sns) = resolve_sns_lookup(&lookup_request, source)?;
    let include_status = request
        .status
        .governance_status_code()
        .into_iter()
        .collect::<Vec<_>>();
    let proposals = source.fetch_sns_proposals(
        &fetch_request,
        &sns,
        request.limit,
        request.before_proposal_id,
        &include_status,
    )?;
    Ok(sns_proposals_report_from_parts(SnsProposalsReportParts {
        list,
        id,
        sns,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        verbose: request.verbose,
        proposals,
    }))
}

fn build_sns_neurons_report_with_source(
    request: &SnsNeuronsRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsReport, SnsHostError> {
    if request.sort.uses_cache() {
        return neurons_cache::build_sns_neurons_report_from_cache(request);
    }

    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let (fetch_request, list, id, sns) = resolve_sns_lookup(&lookup_request, source)?;
    let neurons = source.fetch_sns_neurons(
        &fetch_request,
        &sns,
        request.limit,
        request.owner_principal_id.as_deref(),
    )?;
    Ok(sns_neurons_report_from_parts(SnsNeuronsLiveReportParts {
        list,
        id,
        sns,
        requested_limit: request.limit,
        owner_principal_id: request.owner_principal_id.clone(),
        sort: request.sort,
        verbose: request.verbose,
        neurons,
    }))
}

fn resolve_sns_lookup(
    request: &SnsLookupRequest,
    source: &dyn SnsListSource,
) -> Result<(SnsFetchRequest, MainnetSnsList, usize, MainnetSns), SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetch_request = fetch_request_from_parts(
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    );
    let mut list = source.fetch_deployed_snses(&fetch_request)?;
    assign_sns_ids_in_current_order(&mut list.sns_instances);
    sort_mainnet_sns_instances(&mut list.sns_instances, SnsListSort::Id);
    let (id, sns) = resolve_sns(&list.sns_instances, &request.input)?;
    Ok((fetch_request, list, id, sns))
}

fn lookup_request_from_parts(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    input: &str,
) -> SnsLookupRequest {
    SnsLookupRequest {
        network: network.to_string(),
        source_endpoint: source_endpoint.to_string(),
        now_unix_secs,
        input: input.to_string(),
    }
}

struct SnsNeuronsLiveReportParts {
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    requested_limit: u32,
    owner_principal_id: Option<String>,
    sort: SnsNeuronsSort,
    verbose: bool,
    neurons: MainnetSnsNeurons,
}

struct SnsProposalReportParts {
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    proposal_id: u64,
    verbose: bool,
    proposal: MainnetSnsProposal,
}

struct SnsProposalsReportParts {
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    requested_limit: u32,
    before_proposal_id: Option<u64>,
    status: SnsProposalStatusFilter,
    verbose: bool,
    proposals: MainnetSnsProposals,
}

fn sns_list_report_from_list(
    list: MainnetSnsList,
    verbose: bool,
    sort: SnsListSort,
) -> SnsListReport {
    let MainnetSnsList {
        network,
        sns_wasm_canister_id,
        fetched_at,
        fetched_by,
        source_endpoint,
        sns_instances,
    } = list;
    let metadata_error_count = sns_instances
        .iter()
        .filter(|sns| sns.metadata_error.is_some())
        .count();
    let sns_instances = sns_instances
        .into_iter()
        .map(|sns| SnsListRow {
            id: sns.id,
            name: sns.name,
            root_canister_id: sns.root_canister_id,
            governance_canister_id: sns.governance_canister_id,
            ledger_canister_id: sns.ledger_canister_id,
            swap_canister_id: sns.swap_canister_id,
            index_canister_id: sns.index_canister_id,
            metadata_error: sns.metadata_error,
        })
        .collect::<Vec<_>>();
    SnsListReport {
        schema_version: SNS_LIST_REPORT_SCHEMA_VERSION,
        network,
        sns_wasm_canister_id,
        fetched_at,
        source_endpoint,
        fetched_by,
        verbose,
        sort: sort.as_str().to_string(),
        sns_count: sns_instances.len(),
        metadata_error_count,
        sns_instances,
    }
}

fn sns_info_report_from_list(list: MainnetSnsList, id: usize, sns: MainnetSns) -> SnsInfoReport {
    SnsInfoReport {
        schema_version: SNS_INFO_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        description: sns.description,
        url: sns.url,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        swap_canister_id: sns.swap_canister_id,
        index_canister_id: sns.index_canister_id,
        metadata_error: sns.metadata_error,
    }
}

fn sns_token_report_from_parts(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    token: MainnetSnsToken,
) -> SnsTokenReport {
    SnsTokenReport {
        schema_version: SNS_TOKEN_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        sns_index_canister_id: sns.index_canister_id,
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        ledger_index_canister_id: token.ledger_index_canister_id,
        ledger_index_error: token.ledger_index_error,
        supported_standards: token.supported_standards,
        metadata: token.metadata,
    }
}

fn sns_params_report_from_parts(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    parameters: SnsGovernanceParameters,
) -> SnsParamsReport {
    SnsParamsReport {
        schema_version: SNS_PARAMS_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        parameters,
    }
}

fn sns_proposal_report_from_parts(parts: SnsProposalReportParts) -> SnsProposalReport {
    SnsProposalReport {
        schema_version: SNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        proposal_id: parts.proposal_id,
        verbose: parts.verbose,
        proposal: parts.proposal.proposal,
    }
}

fn sns_proposals_report_from_parts(parts: SnsProposalsReportParts) -> SnsProposalsReport {
    let proposal_count = parts.proposals.proposals.len();
    SnsProposalsReport {
        schema_version: SNS_PROPOSALS_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        before_proposal_id: parts.before_proposal_id,
        status_filter: parts.status.as_str().to_string(),
        verbose: parts.verbose,
        proposal_count,
        proposals: parts.proposals.proposals,
    }
}

fn sns_neurons_report_from_parts(parts: SnsNeuronsLiveReportParts) -> SnsNeuronsReport {
    let neuron_count = parts.neurons.neurons.len();
    SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        owner_principal_id: parts.owner_principal_id,
        verbose: parts.verbose,
        data_source: "live".to_string(),
        sort: parts.sort.as_str().to_string(),
        cache_path: None,
        cache_complete: None,
        total_neuron_count: neuron_count,
        neuron_count,
        neurons: parts.neurons.neurons,
    }
}

fn assign_sns_ids_in_current_order(instances: &mut [MainnetSns]) {
    for (index, sns) in instances.iter_mut().enumerate() {
        sns.id = index + 1;
    }
}

fn sort_mainnet_sns_instances(instances: &mut [MainnetSns], sort: SnsListSort) {
    match sort {
        SnsListSort::Id => sort_mainnet_sns_instances_by_id(instances),
        SnsListSort::Name => instances.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.cmp(&right.id))
        }),
    }
}

fn sort_mainnet_sns_instances_by_id(instances: &mut [MainnetSns]) {
    instances.sort_by_key(|sns| sns.id);
}

fn resolve_sns(instances: &[MainnetSns], input: &str) -> Result<(usize, MainnetSns), SnsHostError> {
    if let Ok(id) = input.parse::<usize>() {
        return instances
            .iter()
            .find(|sns| sns.id == id)
            .cloned()
            .map(|sns| (id, sns))
            .ok_or(SnsHostError::UnknownSnsId {
                id,
                sns_count: instances.len(),
            });
    }

    let root_canister_id = Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })?
        .to_text();
    instances
        .iter()
        .find(|sns| sns.root_canister_id == root_canister_id)
        .map(|sns| (sns.id, sns.clone()))
        .ok_or(SnsHostError::UnknownSnsRoot { root_canister_id })
}

fn fetch_request_from_parts(
    source_endpoint: &str,
    now_unix_secs: u64,
    fetched_by: String,
) -> SnsFetchRequest {
    SnsFetchRequest {
        endpoint: source_endpoint.to_string(),
        fetched_at: format_utc_timestamp_secs(now_unix_secs),
        fetched_by,
    }
}

fn enforce_mainnet_network(network: &str) -> Result<(), SnsHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(SnsHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}

pub(super) fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}

pub(super) fn hex_bytes(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests;
