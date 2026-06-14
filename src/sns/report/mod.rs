use crate::{hex::hex_bytes, ic_registry::DEFAULT_MAINNET_ENDPOINT};
use assemble::{
    SnsNeuronsLiveReportParts, SnsProposalReportParts, SnsProposalsReportParts,
    sns_info_report_from_list, sns_list_report_from_list, sns_neurons_report_from_parts,
    sns_params_report_from_parts, sns_proposal_report_from_parts, sns_proposals_report_from_parts,
    sns_token_report_from_parts,
};
use live::LiveSnsSource;
use lookup::{
    assign_sns_ids_in_current_order, enforce_mainnet_network, lookup_request_from_parts,
    resolve_sns_lookup, sns_list_fetch_request, sort_mainnet_sns_instances,
};
pub use model::*;
use source::{
    MainnetSns, MainnetSnsCanisters, MainnetSnsList, MainnetSnsNeuronPage, MainnetSnsNeurons,
    MainnetSnsProposal, MainnetSnsProposals, MainnetSnsToken, SnsFetchRequest, SnsListSource,
    SnsNeuronId, SnsNeuronsSource, SnsParamsSource, SnsProposalSource, SnsProposalsSource,
    SnsTokenSource,
};

mod assemble;
mod live;
mod lookup;
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

#[cfg(test)]
use crate::subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs};

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
    build_sns_list_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_info_report(request: &SnsInfoRequest) -> Result<SnsInfoReport, SnsHostError> {
    build_sns_info_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_params_report(
    request: &SnsParamsRequest,
) -> Result<SnsParamsReport, SnsHostError> {
    build_sns_params_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_token_report(request: &SnsTokenRequest) -> Result<SnsTokenReport, SnsHostError> {
    build_sns_token_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_proposal_report(
    request: &SnsProposalRequest,
) -> Result<SnsProposalReport, SnsHostError> {
    build_sns_proposal_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_proposals_report(
    request: &SnsProposalsRequest,
) -> Result<SnsProposalsReport, SnsHostError> {
    build_sns_proposals_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_neurons_report(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    build_sns_neurons_report_with_source(request, &LiveSnsSource)
}

fn build_sns_list_report_with_source(
    request: &SnsListRequest,
    source: &dyn SnsListSource,
) -> Result<SnsListReport, SnsHostError> {
    let fetch_request = sns_list_fetch_request(request)?;
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
    let lookup = resolve_sns_lookup(request, source)?;
    Ok(sns_info_report_from_list(
        lookup.list,
        lookup.id,
        lookup.sns,
    ))
}

fn build_sns_params_report_with_source(
    request: &SnsParamsRequest,
    source: &dyn SnsParamsSource,
) -> Result<SnsParamsReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    let parameters = source.fetch_sns_params(&lookup.fetch_request, &lookup.sns)?;
    Ok(sns_params_report_from_parts(
        lookup.list,
        lookup.id,
        lookup.sns,
        parameters,
    ))
}

fn build_sns_token_report_with_source(
    request: &SnsTokenRequest,
    source: &dyn SnsTokenSource,
) -> Result<SnsTokenReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    let token = source.fetch_sns_token(&lookup.fetch_request, &lookup.sns)?;
    Ok(sns_token_report_from_parts(
        lookup.list,
        lookup.id,
        lookup.sns,
        token,
    ))
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
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let proposal =
        source.fetch_sns_proposal(&lookup.fetch_request, &lookup.sns, request.proposal_id)?;
    Ok(sns_proposal_report_from_parts(SnsProposalReportParts {
        list: lookup.list,
        id: lookup.id,
        sns: lookup.sns,
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
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let include_status = request
        .status
        .governance_status_code()
        .into_iter()
        .collect::<Vec<_>>();
    let proposals = source.fetch_sns_proposals(
        &lookup.fetch_request,
        &lookup.sns,
        request.limit,
        request.before_proposal_id,
        &include_status,
    )?;
    Ok(sns_proposals_report_from_parts(SnsProposalsReportParts {
        list: lookup.list,
        id: lookup.id,
        sns: lookup.sns,
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
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let neurons = source.fetch_sns_neurons(
        &lookup.fetch_request,
        &lookup.sns,
        request.limit,
        request.owner_principal_id.as_deref(),
    )?;
    Ok(sns_neurons_report_from_parts(SnsNeuronsLiveReportParts {
        list: lookup.list,
        id: lookup.id,
        sns: lookup.sns,
        requested_limit: request.limit,
        owner_principal_id: request.owner_principal_id.clone(),
        sort: request.sort,
        verbose: request.verbose,
        neurons,
    }))
}

pub(super) fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}

#[cfg(test)]
mod tests;
