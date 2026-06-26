mod error;
#[cfg(feature = "host")]
mod host;
mod json;
mod model;
mod principal;
#[cfg(feature = "host")]
mod report;
mod resolver;
#[cfg(feature = "host")]
mod text;
#[cfg(feature = "host")]
mod time;

pub use error::CatalogError;
#[cfg(feature = "host")]
pub(crate) use host::{
    LiveNnsRegistryRefreshSource, SubnetCatalogCacheRequest, SubnetCatalogHostError,
    SubnetCatalogRefreshRequest, SubnetCatalogRefreshSource, load_or_refresh_subnet_catalog,
    refresh_subnet_catalog,
};
#[cfg(all(test, feature = "host"))]
pub(crate) use host::{
    load_cached_subnet_catalog, refresh_subnet_catalog_with_source, subnet_catalog_path,
    subnet_catalog_refresh_lock_path,
};
pub use json::{catalog_to_pretty_json, parse_catalog_json};
pub use model::{
    ClassificationSource, GeographicScope, RoutingRange, SubnetCatalog, SubnetInfo, SubnetKind,
    SubnetSpecialization,
};
pub use principal::canonical_principal_text;
pub(crate) use principal::{parse_principal, principal_bytes};
#[cfg(feature = "host")]
pub(crate) use report::{
    CatalogStaleStatus, SubnetCatalogFilters, SubnetCatalogInfoReport, SubnetCatalogInfoRequest,
    SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogRefreshReport,
    build_subnet_catalog_info_report, build_subnet_catalog_list_report,
};
#[cfg(all(test, feature = "host"))]
pub(crate) use report::{SubnetCatalogSubnetRow, build_subnet_catalog_list_report_with_source};
pub use resolver::{ResolveAs, ResolvedSubnet, ResolvedSubnetSubject};
#[cfg(all(test, feature = "host"))]
pub(crate) use text::compact_principal;
#[cfg(feature = "host")]
pub(crate) use text::{
    subnet_catalog_info_report_text, subnet_catalog_list_report_text,
    subnet_catalog_list_report_verbose_text, subnet_catalog_refresh_report_text,
};
#[cfg(all(test, feature = "host"))]
pub(crate) use time::parse_stale_after_duration;
#[cfg(feature = "host")]
pub(crate) use time::{catalog_stale_status, format_utc_timestamp_secs};

pub const CATALOG_SCHEMA_VERSION: u32 = 1;
pub const MAINNET_NETWORK: &str = "ic";
pub const MAINNET_REGISTRY_CANISTER_ID: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
#[cfg(feature = "host")]
pub(crate) const DEFAULT_STALE_AFTER_SECONDS: u64 = 7 * 24 * 60 * 60;
#[cfg(feature = "host")]
pub(crate) const DEFAULT_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "host")]
pub(crate) const DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "host")]
pub(crate) const SUBNET_CATALOG_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub(crate) const SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub(crate) const SUBNET_CATALOG_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;

#[cfg(test)]
mod core_tests;
#[cfg(all(test, feature = "host"))]
mod tests;
