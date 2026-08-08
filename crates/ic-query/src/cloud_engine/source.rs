//! Module: cloud_engine::source
//!
//! Responsibility: define CloudEngine source capabilities and the bounded native adapter.
//! Does not own: report validation, CLI parsing, caching, or rendering.
//! Boundary: operator binding uses exactly one query, operator detail uses at most five,
//! and marketplace collection uses exactly two.

use super::{
    CloudEngineHostError, CloudEngineOperatorBindingSourceData, CloudEngineOperatorSourceData,
    CloudEnginePricesSourceData, MAINNET_CLOUD_ENGINE_CANISTER_ID, enforce_mainnet_network,
    wire::{
        CloudEngineMarketplaceEntryWire, GetCaffeineSettingsResult, GetEngineOperatorBySubnetArgs,
        GetEngineOperatorBySubnetResult, GetEngineOwnerResult, GetPlatformAdminResult,
        ListDomainsResult,
    },
};
use crate::{
    agent::build_ic_agent, runtime::block_on_current_thread,
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::{CandidType, Principal};
use serde::Deserialize;

const GET_ENGINE_OPERATOR_BY_SUBNET_METHOD: &str = "getEngineOperatorBySubnet";
const GET_ENGINE_OWNER_METHOD: &str = "getEngineOwner";
const GET_PLATFORM_ADMIN_METHOD: &str = "getPlatformAdmin";
const GET_CAFFEINE_SETTINGS_METHOD: &str = "getCaffeineSettings";
const LIST_DOMAINS_METHOD: &str = "listDomains";
const GET_NETWORK_FEE_METHOD: &str = "getNetworkFee";
const LIST_MARKETPLACE_PRICES_METHOD: &str = "listMarketplacePrices";

///
/// CloudEngineSourceRequest
///
/// Network and collection provenance for direct CloudEngine queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineSourceRequest {
    /// Network to query.
    pub network: String,
    /// Replica endpoint used for the queries.
    pub endpoint: String,
    /// UTC collection timestamp recorded in the report.
    pub fetched_at: String,
    /// Collector identity recorded in the report.
    pub fetched_by: String,
}

impl CloudEngineSourceRequest {
    /// Create source settings for one CloudEngine report.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }

    /// Create source settings from a Unix collection timestamp.
    #[must_use]
    pub fn from_unix_secs(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at_unix_secs: u64,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self::new(
            network,
            endpoint,
            format_utc_timestamp_secs(fetched_at_unix_secs),
            fetched_by,
        )
    }
}

///
/// CloudEngineSource
///
/// Source capabilities for public CloudEngine operator and marketplace data.
///

pub trait CloudEngineSource {
    /// Resolve one Subnet and fetch its public operator details when registered.
    fn fetch_operator(
        &self,
        request: &CloudEngineSourceRequest,
        subnet_id: &str,
    ) -> Result<CloudEngineOperatorSourceData, CloudEngineHostError>;

    /// Fetch the public network fee and complete bounded marketplace price response.
    fn fetch_prices(
        &self,
        request: &CloudEngineSourceRequest,
    ) -> Result<CloudEnginePricesSourceData, CloudEngineHostError>;
}

///
/// CloudEngineOperatorBindingSource
///
/// Focused source capability for one public Subnet-to-operator lookup without details.
///

pub trait CloudEngineOperatorBindingSource {
    /// Resolve exactly one Subnet through the CloudEngine control plane.
    fn fetch_operator_binding(
        &self,
        request: &CloudEngineSourceRequest,
        subnet_id: &str,
    ) -> Result<CloudEngineOperatorBindingSourceData, CloudEngineHostError>;
}

///
/// LiveCloudEngineSource
///
/// Built-in live adapter for the mainnet CloudEngine control-plane canister.
///

#[derive(Clone, Copy, Debug, Default)]
pub struct LiveCloudEngineSource;

impl CloudEngineSource for LiveCloudEngineSource {
    fn fetch_operator(
        &self,
        request: &CloudEngineSourceRequest,
        subnet_id: &str,
    ) -> Result<CloudEngineOperatorSourceData, CloudEngineHostError> {
        enforce_mainnet_network(&request.network)?;
        let subnet = parse_principal("subnet_id", subnet_id)?;
        block_on_current_thread(fetch_live_operator(request, subnet))?
    }

    fn fetch_prices(
        &self,
        request: &CloudEngineSourceRequest,
    ) -> Result<CloudEnginePricesSourceData, CloudEngineHostError> {
        enforce_mainnet_network(&request.network)?;
        block_on_current_thread(fetch_live_prices(request))?
    }
}

impl CloudEngineOperatorBindingSource for LiveCloudEngineSource {
    fn fetch_operator_binding(
        &self,
        request: &CloudEngineSourceRequest,
        subnet_id: &str,
    ) -> Result<CloudEngineOperatorBindingSourceData, CloudEngineHostError> {
        enforce_mainnet_network(&request.network)?;
        let subnet = parse_principal("subnet_id", subnet_id)?;
        block_on_current_thread(fetch_live_operator_binding(request, subnet))?
    }
}

async fn fetch_live_operator_binding(
    request: &CloudEngineSourceRequest,
    subnet: Principal,
) -> Result<CloudEngineOperatorBindingSourceData, CloudEngineHostError> {
    let (agent, engine_canister) = live_agent_and_canister(request)?;
    let operator_canister_id = resolve_operator(&agent, &engine_canister, subnet).await?;
    Ok(CloudEngineOperatorBindingSourceData {
        source: request.clone(),
        subnet_id: subnet.to_text(),
        operator_canister_id: operator_canister_id.map(|principal| principal.to_text()),
        query_call_count: 1,
    })
}

async fn fetch_live_operator(
    request: &CloudEngineSourceRequest,
    subnet: Principal,
) -> Result<CloudEngineOperatorSourceData, CloudEngineHostError> {
    let (agent, engine_canister) = live_agent_and_canister(request)?;
    let resolved = resolve_operator(&agent, &engine_canister, subnet).await?;
    let Some(operator) = resolved else {
        return Ok(CloudEngineOperatorSourceData {
            source: request.clone(),
            subnet_id: subnet.to_text(),
            operator_canister_id: None,
            engine_owner: None,
            platform_admin: None,
            caffeine_enabled: None,
            claimed_domains: None,
            query_call_count: 1,
        });
    };

    let empty_arg = empty_args()?;
    let owner: GetEngineOwnerResult = query_candid(
        &agent,
        &operator,
        GET_ENGINE_OWNER_METHOD,
        empty_arg.clone(),
        "GetEngineOwnerResult",
    )
    .await?;
    let platform: GetPlatformAdminResult = query_candid(
        &agent,
        &operator,
        GET_PLATFORM_ADMIN_METHOD,
        empty_arg.clone(),
        "GetPlatformAdminResult",
    )
    .await?;
    let caffeine: GetCaffeineSettingsResult = query_candid(
        &agent,
        &operator,
        GET_CAFFEINE_SETTINGS_METHOD,
        empty_arg.clone(),
        "GetCaffeineSettingsResult",
    )
    .await?;
    let domains: ListDomainsResult = query_candid(
        &agent,
        &operator,
        LIST_DOMAINS_METHOD,
        empty_arg,
        "ListDomainsResult",
    )
    .await?;

    Ok(CloudEngineOperatorSourceData {
        source: request.clone(),
        subnet_id: subnet.to_text(),
        operator_canister_id: Some(operator.to_text()),
        engine_owner: owner.engine_owner.map(|principal| principal.to_text()),
        platform_admin: platform.platform_admin.map(|principal| principal.to_text()),
        caffeine_enabled: caffeine.settings.and_then(|settings| settings.enabled),
        claimed_domains: domains.domains,
        query_call_count: 5,
    })
}

async fn resolve_operator(
    agent: &ic_agent::Agent,
    engine_canister: &Principal,
    subnet: Principal,
) -> Result<Option<Principal>, CloudEngineHostError> {
    let resolve_arg = encode_one(
        &GetEngineOperatorBySubnetArgs {
            subnet_id: Some(subnet),
        },
        "GetEngineOperatorBySubnetArgs",
    )?;
    let resolved: GetEngineOperatorBySubnetResult = query_candid(
        agent,
        engine_canister,
        GET_ENGINE_OPERATOR_BY_SUBNET_METHOD,
        resolve_arg,
        "GetEngineOperatorBySubnetResult",
    )
    .await?;
    Ok(resolved.engine_operator_id)
}

async fn fetch_live_prices(
    request: &CloudEngineSourceRequest,
) -> Result<CloudEnginePricesSourceData, CloudEngineHostError> {
    let (agent, engine_canister) = live_agent_and_canister(request)?;
    let empty_arg = empty_args()?;
    let network_fee: f64 = query_candid(
        &agent,
        &engine_canister,
        GET_NETWORK_FEE_METHOD,
        empty_arg.clone(),
        "NetworkFee",
    )
    .await?;
    let prices: Vec<CloudEngineMarketplaceEntryWire> = query_candid(
        &agent,
        &engine_canister,
        LIST_MARKETPLACE_PRICES_METHOD,
        empty_arg,
        "MarketplaceEntry vector",
    )
    .await?;

    Ok(CloudEnginePricesSourceData {
        source: request.clone(),
        network_fee,
        prices: prices
            .into_iter()
            .map(CloudEngineMarketplaceEntryWire::into_report_row)
            .collect(),
        query_call_count: 2,
    })
}

fn live_agent_and_canister(
    request: &CloudEngineSourceRequest,
) -> Result<(ic_agent::Agent, Principal), CloudEngineHostError> {
    let agent = build_ic_agent(&request.endpoint, |reason| {
        CloudEngineHostError::AgentBuild {
            endpoint: request.endpoint.clone(),
            reason,
        }
    })?;
    let canister = Principal::from_text(MAINNET_CLOUD_ENGINE_CANISTER_ID).map_err(|error| {
        CloudEngineHostError::CanisterId {
            reason: error.to_string(),
        }
    })?;
    Ok((agent, canister))
}

fn parse_principal(field: &'static str, value: &str) -> Result<Principal, CloudEngineHostError> {
    Principal::from_text(value).map_err(|error| CloudEngineHostError::InvalidPrincipal {
        field,
        reason: error.to_string(),
    })
}

fn empty_args() -> Result<Vec<u8>, CloudEngineHostError> {
    candid::encode_args(()).map_err(|error| CloudEngineHostError::CandidEncode {
        message: "()",
        reason: error.to_string(),
    })
}

fn encode_one<T>(value: &T, message: &'static str) -> Result<Vec<u8>, CloudEngineHostError>
where
    T: CandidType,
{
    candid::encode_one(value).map_err(|error| CloudEngineHostError::CandidEncode {
        message,
        reason: error.to_string(),
    })
}

async fn query_candid<T>(
    agent: &ic_agent::Agent,
    canister: &Principal,
    method: &'static str,
    arg: Vec<u8>,
    response_name: &'static str,
) -> Result<T, CloudEngineHostError>
where
    T: CandidType + for<'de> Deserialize<'de>,
{
    let bytes = agent
        .query(canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|error| CloudEngineHostError::AgentCall {
            method,
            reason: error.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|error| CloudEngineHostError::CandidDecode {
        message: response_name,
        reason: error.to_string(),
    })
}
