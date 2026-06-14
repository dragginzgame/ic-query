mod host;
mod model;
mod report;
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
pub(crate) use report::{
    CatalogStaleStatus, SubnetCatalogFilters, SubnetCatalogInfoReport, SubnetCatalogInfoRequest,
    SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogRefreshReport,
    build_subnet_catalog_info_report, build_subnet_catalog_list_report,
};
#[cfg(test)]
pub(crate) use report::{SubnetCatalogSubnetRow, build_subnet_catalog_list_report_with_source};
pub use resolver::{ResolveAs, ResolvedSubnet, ResolvedSubnetSubject};
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

#[cfg(test)]
mod core_tests;
#[cfg(test)]
mod tests;
