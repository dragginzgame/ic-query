mod host;
mod model;
mod resolver;
mod text;
mod time;

use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
use candid::Principal;
pub(crate) use host::{
    LiveNnsRegistryRefreshSource, SubnetCatalogCacheRequest, SubnetCatalogHostError,
    SubnetCatalogRefreshRequest, SubnetCatalogRefreshSource, load_or_refresh_subnet_catalog,
    refresh_subnet_catalog,
};
#[cfg(test)]
pub(crate) use host::{
    load_cached_subnet_catalog, refresh_subnet_catalog_with_source, subnet_catalog_path,
    subnet_catalog_refresh_lock_path,
};
pub use model::{
    ClassificationSource, GeographicScope, RoutingRange, SubnetCatalog, SubnetInfo, SubnetKind,
    SubnetSpecialization,
};
pub use resolver::{ResolveAs, ResolvedSubnet, ResolvedSubnetSubject};
use serde::{Deserialize, Serialize};
#[cfg(test)]
pub(crate) use text::compact_principal;
pub(crate) use text::{
    subnet_catalog_info_report_text, subnet_catalog_list_report_text,
    subnet_catalog_list_report_verbose_text, subnet_catalog_refresh_report_text,
};
use thiserror::Error as ThisError;
#[cfg(test)]
pub(crate) use time::parse_stale_after_duration;
pub(crate) use time::{catalog_stale_status, format_utc_timestamp_secs};

pub const CATALOG_SCHEMA_VERSION: u32 = 1;
pub const MAINNET_NETWORK: &str = "ic";
pub const MAINNET_REGISTRY_CANISTER_ID: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
pub(crate) const DEFAULT_STALE_AFTER_SECONDS: u64 = 7 * 24 * 60 * 60;
pub(crate) const DEFAULT_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub(crate) const DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub(crate) const SUBNET_CATALOG_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub(crate) const SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub(crate) const SUBNET_CATALOG_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const BASE_13_NODE_CYCLES_PER_BILLION_INSTRUCTIONS: u128 = 1_000_000_000;
const FORMULA_VERSION: &str = "base_13_node_linear_v1";

///
/// CatalogError
///
#[derive(Debug, ThisError)]
pub enum CatalogError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("unsupported subnet catalog schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },

    #[error("subnet catalog must contain at least one subnet")]
    EmptySubnets,

    #[error("subnet catalog must contain at least one routing range")]
    EmptyRoutingRanges,

    #[error("invalid principal in {field}: {value}: {reason}")]
    InvalidPrincipal {
        field: &'static str,
        value: String,
        reason: String,
    },

    #[error("duplicate subnet principal in catalog: {subnet_principal}")]
    DuplicateSubnet { subnet_principal: String },

    #[error("routing range references unknown subnet: {subnet_principal}")]
    UnknownRoutingSubnet { subnet_principal: String },

    #[error(
        "invalid routing range for {subnet_principal}: start {start_canister_id} sorts after end {end_canister_id}"
    )]
    InvalidRoutingRange {
        subnet_principal: String,
        start_canister_id: String,
        end_canister_id: String,
    },

    #[error("subnet principal {subnet_principal} was not found in the cached catalog")]
    UnknownSubnet { subnet_principal: String },

    #[error("principal prefix {prefix:?} did not match cached subnet principals")]
    PrincipalPrefixNotFound { prefix: String },

    #[error("principal prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousPrincipalPrefix {
        prefix: String,
        matches: Vec<String>,
    },

    #[error(
        "canister principal {canister_principal} was not covered by cached routing ranges at registry_version={registry_version}, catalog_schema_version={catalog_schema_version}"
    )]
    RouteNotFound {
        canister_principal: String,
        registry_version: u64,
        catalog_schema_version: u32,
    },
}

/// Decode and validate one subnet catalog JSON payload.
pub fn parse_catalog_json(data: &str) -> Result<SubnetCatalog, CatalogError> {
    let catalog = serde_json::from_str::<SubnetCatalog>(data)?;
    catalog.validate()?;
    Ok(catalog)
}

/// Render one subnet catalog JSON payload with stable pretty formatting.
pub fn catalog_to_pretty_json(catalog: &SubnetCatalog) -> Result<String, CatalogError> {
    Ok(serde_json::to_string_pretty(catalog)?)
}

/// Parse a textual IC principal into canonical text.
pub fn canonical_principal_text(value: &str) -> Result<String, CatalogError> {
    Ok(parse_principal(value, "principal")?.to_text())
}

pub(crate) fn parse_principal(value: &str, field: &'static str) -> Result<Principal, CatalogError> {
    Principal::from_text(value).map_err(|err| CatalogError::InvalidPrincipal {
        field,
        value: value.to_string(),
        reason: err.to_string(),
    })
}

pub(crate) fn principal_bytes(value: &str, field: &'static str) -> Result<Vec<u8>, CatalogError> {
    Ok(parse_principal(value, field)?.as_slice().to_vec())
}

///
/// SubnetCatalogFilters
///
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SubnetCatalogFilters {
    pub kind: Option<SubnetKind>,
    pub specialization: Option<SubnetSpecialization>,
    pub geographic_scope: Option<GeographicScope>,
}

///
/// SubnetCatalogListRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubnetCatalogListRequest {
    pub cache: SubnetCatalogCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub stale_after_seconds: u64,
    pub filters: SubnetCatalogFilters,
    pub show_ranges: bool,
    pub range_limit: usize,
    pub range_offset: usize,
}

///
/// SubnetCatalogInfoRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubnetCatalogInfoRequest {
    pub cache: SubnetCatalogCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub forced: Option<ResolveAs>,
    pub now_unix_secs: u64,
    pub stale_after_seconds: u64,
}

///
/// CatalogStaleStatus
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CatalogStaleStatus {
    pub catalog_stale: bool,
    pub stale_reason: String,
    pub stale_after_seconds: u64,
    pub fetched_at_unix_secs: Option<u64>,
    pub age_seconds: Option<u64>,
}

///
/// SubnetCatalogListReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SubnetCatalogListReport {
    pub schema_version: u32,
    pub network: String,
    pub catalog_path: String,
    pub catalog_schema_version: u32,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub catalog_stale: bool,
    pub stale_reason: String,
    pub resolver_backend: String,
    pub subnets: Vec<SubnetCatalogSubnetRow>,
}

///
/// SubnetCatalogSubnetRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SubnetCatalogSubnetRow {
    pub subnet_principal: String,
    pub subnet_kind: SubnetKind,
    pub subnet_kind_source: ClassificationSource,
    pub subnet_specialization: SubnetSpecialization,
    pub subnet_specialization_source: ClassificationSource,
    pub geographic_scope: GeographicScope,
    pub geographic_scope_source: ClassificationSource,
    pub subnet_label: String,
    pub subnet_label_source: ClassificationSource,
    pub node_count: Option<u32>,
    pub charges_apply_by_default: bool,
    pub range_count: usize,
    pub ranges_shown: usize,
    pub range_offset: usize,
    pub range_limit: usize,
    pub ranges: Vec<RoutingRange>,
}

///
/// SubnetCatalogInfoReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SubnetCatalogInfoReport {
    pub schema_version: u32,
    pub input_principal: String,
    pub resolved_as: String,
    pub resolved_from: String,
    pub subnet_principal: String,
    pub subnet_kind: SubnetKind,
    pub subnet_kind_source: ClassificationSource,
    pub subnet_specialization: SubnetSpecialization,
    pub subnet_specialization_source: ClassificationSource,
    pub geographic_scope: GeographicScope,
    pub geographic_scope_source: ClassificationSource,
    pub subnet_label: String,
    pub subnet_label_source: ClassificationSource,
    pub node_count: Option<u32>,
    pub charges_apply_to_subject: bool,
    pub charge_applicability_reason: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub catalog_schema_version: u32,
    pub catalog_path: String,
    pub fetched_at: String,
    pub catalog_stale: bool,
    pub stale_reason: String,
    pub resolver_backend: String,
    pub matched_canister_principal: Option<String>,
    pub matched_routing_range: Option<RoutingRange>,
    pub cycles_per_billion_instructions: Option<u128>,
    pub rate_source: Option<String>,
    pub formula_version: Option<String>,
}

///
/// SubnetCatalogRefreshReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SubnetCatalogRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub catalog_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub dry_run: bool,
    pub wrote_catalog: bool,
    pub replaced_existing_catalog: bool,
    pub subnet_count: usize,
    pub routing_range_count: usize,
}

pub(crate) fn build_subnet_catalog_list_report(
    request: &SubnetCatalogListRequest,
) -> Result<SubnetCatalogListReport, SubnetCatalogHostError> {
    build_subnet_catalog_list_report_with_source(request, &LiveNnsRegistryRefreshSource)
}

fn build_subnet_catalog_list_report_with_source(
    request: &SubnetCatalogListRequest,
    source: &dyn SubnetCatalogRefreshSource,
) -> Result<SubnetCatalogListReport, SubnetCatalogHostError> {
    let cached = load_or_refresh_subnet_catalog(
        &request.cache,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let stale = catalog_stale_status(
        &cached.catalog,
        request.now_unix_secs,
        request.stale_after_seconds,
    );
    let subnets = cached
        .catalog
        .subnets
        .iter()
        .filter(|subnet| subnet_matches_filters(subnet, request.filters))
        .map(|subnet| subnet_row(&cached.catalog, subnet, request))
        .collect::<Vec<_>>();

    Ok(SubnetCatalogListReport {
        schema_version: SUBNET_CATALOG_LIST_REPORT_SCHEMA_VERSION,
        network: cached.catalog.network,
        catalog_path: cached.path.display().to_string(),
        catalog_schema_version: cached.catalog.catalog_schema_version,
        registry_canister_id: cached.catalog.registry_canister_id,
        registry_version: cached.catalog.registry_version,
        fetched_at: cached.catalog.fetched_at,
        catalog_stale: stale.catalog_stale,
        stale_reason: stale.stale_reason,
        resolver_backend: cached.catalog.resolver_backend,
        subnets,
    })
}

pub(crate) fn build_subnet_catalog_info_report(
    request: &SubnetCatalogInfoRequest,
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    build_subnet_catalog_info_report_with_source(request, &LiveNnsRegistryRefreshSource)
}

fn build_subnet_catalog_info_report_with_source(
    request: &SubnetCatalogInfoRequest,
    source: &dyn SubnetCatalogRefreshSource,
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    let cached = load_or_refresh_subnet_catalog(
        &request.cache,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let stale = catalog_stale_status(
        &cached.catalog,
        request.now_unix_secs,
        request.stale_after_seconds,
    );
    let resolved = cached
        .catalog
        .resolve_principal_or_prefix(&request.input, request.forced)?;
    let (charges_apply_to_subject, charge_applicability_reason) =
        charge_applicability(resolved.resolved_as, resolved.subnet.subnet_kind);
    let cycles_per_billion_instructions = catalog_cycles_per_billion(&resolved.subnet);
    let rate_source = cycles_per_billion_instructions
        .is_some()
        .then(|| "nns-registry-cache".to_string());
    let formula_version = cycles_per_billion_instructions
        .is_some()
        .then(|| FORMULA_VERSION.to_string());

    Ok(SubnetCatalogInfoReport {
        schema_version: SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION,
        input_principal: resolved.input_principal,
        resolved_as: resolved.resolved_as.as_str().to_string(),
        resolved_from: resolved.resolved_from,
        subnet_principal: resolved.subnet.subnet_principal,
        subnet_kind: resolved.subnet.subnet_kind,
        subnet_kind_source: resolved.subnet.subnet_kind_source,
        subnet_specialization: resolved.subnet.subnet_specialization,
        subnet_specialization_source: resolved.subnet.subnet_specialization_source,
        geographic_scope: resolved.subnet.geographic_scope,
        geographic_scope_source: resolved.subnet.geographic_scope_source,
        subnet_label: resolved.subnet.subnet_label,
        subnet_label_source: resolved.subnet.subnet_label_source,
        node_count: resolved.subnet.node_count,
        charges_apply_to_subject,
        charge_applicability_reason,
        registry_canister_id: cached.catalog.registry_canister_id,
        registry_version: cached.catalog.registry_version,
        catalog_schema_version: cached.catalog.catalog_schema_version,
        catalog_path: cached.path.display().to_string(),
        fetched_at: cached.catalog.fetched_at,
        catalog_stale: stale.catalog_stale,
        stale_reason: stale.stale_reason,
        resolver_backend: cached.catalog.resolver_backend,
        matched_canister_principal: resolved.matched_canister_principal,
        matched_routing_range: resolved.matched_routing_range,
        cycles_per_billion_instructions,
        rate_source,
        formula_version,
    })
}

fn subnet_matches_filters(subnet: &SubnetInfo, filters: SubnetCatalogFilters) -> bool {
    filters.kind.is_none_or(|kind| subnet.subnet_kind == kind)
        && filters
            .specialization
            .is_none_or(|specialization| subnet.subnet_specialization == specialization)
        && filters
            .geographic_scope
            .is_none_or(|scope| subnet.geographic_scope == scope)
}

fn subnet_row(
    catalog: &SubnetCatalog,
    subnet: &SubnetInfo,
    request: &SubnetCatalogListRequest,
) -> SubnetCatalogSubnetRow {
    let ranges = catalog.routing_ranges_for_subnet(&subnet.subnet_principal);
    let range_count = ranges.len();
    let shown_ranges = if request.show_ranges {
        ranges
            .into_iter()
            .skip(request.range_offset)
            .take(request.range_limit)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    SubnetCatalogSubnetRow {
        subnet_principal: subnet.subnet_principal.clone(),
        subnet_kind: subnet.subnet_kind,
        subnet_kind_source: subnet.subnet_kind_source,
        subnet_specialization: subnet.subnet_specialization,
        subnet_specialization_source: subnet.subnet_specialization_source,
        geographic_scope: subnet.geographic_scope,
        geographic_scope_source: subnet.geographic_scope_source,
        subnet_label: subnet.subnet_label.clone(),
        subnet_label_source: subnet.subnet_label_source,
        node_count: subnet.node_count,
        charges_apply_by_default: subnet.charges_apply_by_default,
        range_count,
        ranges_shown: shown_ranges.len(),
        range_offset: request.range_offset,
        range_limit: request.range_limit,
        ranges: shown_ranges,
    }
}

fn charge_applicability(subject: ResolvedSubnetSubject, kind: SubnetKind) -> (bool, String) {
    match kind {
        SubnetKind::Application | SubnetKind::CloudEngine => {
            (true, "charged_user_canister_subnet".to_string())
        }
        SubnetKind::System if subject == ResolvedSubnetSubject::Subnet => {
            (false, "system_subnet_core_canister".to_string())
        }
        SubnetKind::System => (false, "system_subnet_unknown_subject".to_string()),
        SubnetKind::Unknown => (false, "unknown_subnet_type".to_string()),
    }
}

fn catalog_cycles_per_billion(subnet: &SubnetInfo) -> Option<u128> {
    if !subnet.subnet_kind.charges_apply_by_default() {
        return None;
    }
    let node_count = u128::from(subnet.node_count?);
    if node_count == 0 {
        return None;
    }
    Some(ceil_div(
        BASE_13_NODE_CYCLES_PER_BILLION_INSTRUCTIONS * node_count,
        13,
    ))
}

const fn ceil_div(numerator: u128, denominator: u128) -> u128 {
    numerator.div_ceil(denominator)
}

#[cfg(test)]
#[path = "tests/core.rs"]
mod core_tests;
#[cfg(test)]
mod tests;
