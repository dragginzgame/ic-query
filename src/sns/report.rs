use crate::{
    ic_registry::DEFAULT_MAINNET_ENDPOINT,
    runtime::block_on_current_thread,
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
    table::{ColumnAlign, render_table},
};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use futures::{StreamExt, stream};
use ic_agent::Agent;
use serde::Serialize;
use thiserror::Error as ThisError;

pub const DEFAULT_SNS_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const MAINNET_SNS_WASM_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";

const SNS_LIST_REPORT_SCHEMA_VERSION: u32 = 3;
const SNS_INFO_REPORT_SCHEMA_VERSION: u32 = 2;
const COMPACT_PRINCIPAL_CHARS: usize = 5;
const SNS_METADATA_CONCURRENCY: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsListRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub verbose: bool,
    pub sort: SnsListSort,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsInfoRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsListReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub verbose: bool,
    pub sort: String,
    pub sns_count: usize,
    pub metadata_error_count: usize,
    pub sns_instances: Vec<SnsListRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsListRow {
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsInfoReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsListSort {
    #[default]
    Id,
    Name,
}

impl SnsListSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
    }
}

#[derive(Debug, ThisError)]
pub enum SnsHostError {
    #[error(
        "`icq sns` supports only the mainnet `ic` network\n\nThe SNS list is queried from the public Internet Computer mainnet SNS-W canister.\nLocal replica SNS discovery is not implemented yet.\n\nTry:\n  icq --network ic sns list"
    )]
    UnsupportedNetwork { network: String },

    #[error("failed to create Tokio runtime for SNS query: {0}")]
    Runtime(String),

    #[error("failed to build IC agent for endpoint {endpoint}: {reason}")]
    AgentBuild { endpoint: String, reason: String },

    #[error("invalid {field}: {reason}")]
    InvalidPrincipal { field: &'static str, reason: String },

    #[error("failed to encode Candid request for {message}: {reason}")]
    CandidEncode {
        message: &'static str,
        reason: String,
    },

    #[error("SNS query method {method} failed: {reason}")]
    AgentCall {
        method: &'static str,
        reason: String,
    },

    #[error("failed to decode Candid response {message}: {reason}")]
    CandidDecode {
        message: &'static str,
        reason: String,
    },

    #[error("SNS list id {id} is out of range; list contains {sns_count} deployed SNS instances")]
    UnknownSnsId { id: usize, sns_count: usize },

    #[error("could not find deployed SNS with root principal {root_canister_id}")]
    UnknownSnsRoot { root_canister_id: String },

    #[error("SNS lookup input must be a list id or root principal: {input}")]
    InvalidLookup { input: String },
}

pub fn build_sns_list_report(request: &SnsListRequest) -> Result<SnsListReport, SnsHostError> {
    build_sns_list_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_info_report(request: &SnsInfoRequest) -> Result<SnsInfoReport, SnsHostError> {
    build_sns_info_report_with_source(request, &LiveSnsListSource)
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
    assign_stable_sns_ids(&mut list.sns_instances);
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
    enforce_mainnet_network(&request.network)?;
    let fetch_request = fetch_request_from_parts(
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    );
    let mut list = source.fetch_deployed_snses(&fetch_request)?;
    assign_stable_sns_ids(&mut list.sns_instances);
    sort_mainnet_sns_instances(&mut list.sns_instances, SnsListSort::Id);
    let (id, sns) = resolve_sns(&list.sns_instances, &request.input)?;
    Ok(sns_info_report_from_list(list, id, sns))
}

#[must_use]
pub fn sns_list_report_text(report: &SnsListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("network: {}", report.network));
    lines.push(format!(
        "sns_wasm_canister_id: {}",
        report.sns_wasm_canister_id
    ));
    lines.push(format!("sns_count: {}", report.sns_count));
    lines.push(format!("fetched_at: {}", report.fetched_at));
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("sort: {}", report.sort));
    lines.push(format!("metadata_errors: {}", report.metadata_error_count));
    if !report.sns_instances.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "ID",
                "NAME",
                "ROOT",
                "GOVERNANCE",
                "LEDGER",
                "SWAP",
                "INDEX",
            ],
            &report
                .sns_instances
                .iter()
                .map(|sns| {
                    [
                        sns.id.to_string(),
                        sns.name.clone(),
                        principal_for_list(&sns.root_canister_id, report.verbose),
                        principal_for_list(&sns.governance_canister_id, report.verbose),
                        principal_for_list(&sns.ledger_canister_id, report.verbose),
                        principal_for_list(&sns.swap_canister_id, report.verbose),
                        principal_for_list(&sns.index_canister_id, report.verbose),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }
    if report.verbose && report.metadata_error_count > 0 {
        lines.push(String::new());
        lines.push("metadata_error_details:".to_string());
        for (governance_canister_id, error) in report.sns_instances.iter().filter_map(|sns| {
            sns.metadata_error
                .as_deref()
                .map(|error| (&sns.governance_canister_id, error))
        }) {
            lines.push(format!("- {governance_canister_id}: {error}"));
        }
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_info_report_text(report: &SnsInfoReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!(
            "description: {}",
            optional_text(report.description.as_ref())
        ),
        format!("url: {}", optional_text(report.url.as_ref())),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("swap_canister_id: {}", report.swap_canister_id),
        format!("index_canister_id: {}", report.index_canister_id),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.metadata_error.as_deref() {
        lines.push(format!("metadata_error: {error}"));
    }
    lines.join("\n")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsFetchRequest {
    endpoint: String,
    fetched_at: String,
    fetched_by: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSnsList {
    network: String,
    sns_wasm_canister_id: String,
    fetched_at: String,
    fetched_by: String,
    source_endpoint: String,
    sns_instances: Vec<MainnetSns>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSns {
    id: usize,
    name: String,
    description: Option<String>,
    url: Option<String>,
    root_canister_id: String,
    governance_canister_id: String,
    ledger_canister_id: String,
    swap_canister_id: String,
    index_canister_id: String,
    metadata_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSnsCanisters {
    root_canister_id: String,
    governance_canister_id: String,
    ledger_canister_id: String,
    swap_canister_id: String,
    index_canister_id: String,
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

trait SnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError>;
}

struct LiveSnsListSource;

impl SnsListSource for LiveSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        fetch_mainnet_sns_list(request)
    }
}

fn fetch_mainnet_sns_list(request: &SnsFetchRequest) -> Result<MainnetSnsList, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_list_async(request)).map_err(SnsHostError::Runtime)?
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
    .buffer_unordered(SNS_METADATA_CONCURRENCY)
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

fn assign_stable_sns_ids(instances: &mut [MainnetSns]) {
    instances.sort_by(|left, right| left.root_canister_id.cmp(&right.root_canister_id));
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
    instances.sort_by(|left, right| left.root_canister_id.cmp(&right.root_canister_id));
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
        .enumerate()
        .find(|(_, sns)| sns.root_canister_id == root_canister_id)
        .map(|(index, sns)| (index + 1, sns.clone()))
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

fn principal_from_text(value: &str, field: &'static str) -> Result<Principal, SnsHostError> {
    Principal::from_text(value).map_err(|err| SnsHostError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}

fn principal_for_list(value: &str, verbose: bool) -> String {
    if verbose {
        value.to_string()
    } else {
        short_principal(value)
    }
}

fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}

fn optional_text(value: Option<&String>) -> &str {
    value.map_or("-", String::as_str)
}

fn clean_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
        SnsHostError::UnsupportedNetwork { .. }
        | SnsHostError::Runtime(_)
        | SnsHostError::AgentBuild { .. }
        | SnsHostError::InvalidPrincipal { .. }
        | SnsHostError::UnknownSnsId { .. }
        | SnsHostError::UnknownSnsRoot { .. }
        | SnsHostError::InvalidLookup { .. } => None,
    }
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
