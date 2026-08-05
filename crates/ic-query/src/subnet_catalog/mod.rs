//! Cached NNS subnet catalog models, resolution helpers, builders, and renderers.

mod error;
#[cfg(feature = "subnet-catalog-host")]
mod host;
mod json;
mod model;
mod principal;
#[cfg(feature = "subnet-catalog-host")]
mod report;
mod resolver;
#[cfg(feature = "subnet-catalog-host")]
mod text;
mod time;

pub use error::CatalogError;
#[cfg(feature = "subnet-catalog-host")]
pub use host::{
    CacheDisposition, CatalogAuthorityEvidence, CatalogLoadOutcome, CatalogReadPolicy,
    CatalogSourceSelection, SubnetCatalogCacheRequest, SubnetCatalogErrorCategory,
    SubnetCatalogErrorCode, SubnetCatalogHostError, SubnetCatalogLoadRequest,
    SubnetCatalogRefreshRequest, SubnetCatalogRemediation, SubnetCatalogRetryability,
    SubnetCatalogSource, SubnetCatalogSourceFuture, fetch_subnet_catalog_async,
    load_cached_subnet_catalog, load_subnet_catalog, load_subnet_catalog_async,
    load_subnet_catalog_with_source, load_subnet_catalog_with_source_async, refresh_subnet_catalog,
    refresh_subnet_catalog_async, refresh_subnet_catalog_with_source,
    refresh_subnet_catalog_with_source_async, subnet_catalog_path,
    subnet_catalog_refresh_lock_path,
};
pub use json::{catalog_to_pretty_json, parse_catalog_json};
#[cfg(feature = "subnet-catalog-host")]
pub use model::UncertifiedCatalogCollection;
#[cfg(feature = "subnet-catalog-host")]
pub(in crate::subnet_catalog) use model::catalog_agreement_digest;
pub use model::{
    CLASSIFICATION_SCHEMA_VERSION, CatalogAssurance, CatalogValidationContext,
    ClassificationSource, GeographicScope, RESOLVER_SCHEMA_VERSION, RawSubnetCatalog, RoutingRange,
    SubnetCatalogProvenance, SubnetInfo, SubnetKind, SubnetSpecialization, ValidatedSubnetCatalog,
};
pub use principal::canonical_principal_text;
pub(crate) use principal::{parse_principal, principal_bytes};
#[cfg(feature = "subnet-catalog-host")]
pub use report::{
    CatalogStaleStatus, SubnetCatalogFilters, SubnetCatalogInfoReport, SubnetCatalogInfoRequest,
    SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogRefreshReport,
    SubnetCatalogSubnetRow, build_subnet_catalog_info_report,
    build_subnet_catalog_info_report_with_source, build_subnet_catalog_list_report,
    build_subnet_catalog_list_report_with_source,
};
pub use resolver::{ResolveAs, ResolvedCanisterRoute, ResolvedSubnet, ResolvedSubnetSubject};
#[cfg(feature = "subnet-catalog-host")]
pub use text::{
    subnet_catalog_info_report_text, subnet_catalog_list_report_text,
    subnet_catalog_list_report_verbose_text, subnet_catalog_refresh_report_text,
};
#[cfg(feature = "subnet-catalog-host")]
pub use time::catalog_stale_status;
pub(crate) use time::format_utc_timestamp_secs;
#[cfg(feature = "subnet-catalog-host")]
pub(crate) use time::parse_utc_timestamp_secs;

pub const CATALOG_SCHEMA_VERSION: u32 = 2;
pub const MAINNET_NETWORK: &str = "ic";
pub const MAINNET_REGISTRY_CANISTER_ID: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
#[cfg(feature = "subnet-catalog-host")]
/// Maximum future timestamp skew accepted by default during catalog validation.
pub const DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS: u64 = 5 * 60;
#[cfg(feature = "subnet-catalog-host")]
pub const DEFAULT_STALE_AFTER_SECONDS: u64 = 7 * 24 * 60 * 60;
#[cfg(feature = "subnet-catalog-host")]
pub const DEFAULT_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "subnet-catalog-host")]
pub const DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "subnet-catalog-host")]
/// Minimum number of independent hosts required for endpoint agreement.
pub const MIN_SUBNET_CATALOG_AGREEMENT_ENDPOINTS: usize = 2;
#[cfg(feature = "subnet-catalog-host")]
/// Maximum number of endpoints accepted by one bounded agreement collection.
pub const MAX_SUBNET_CATALOG_AGREEMENT_ENDPOINTS: usize = 3;
#[cfg(feature = "subnet-catalog-host")]
pub(crate) const SUBNET_CATALOG_LIST_REPORT_SCHEMA_VERSION: u32 = 2;
#[cfg(feature = "subnet-catalog-host")]
pub(crate) const SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION: u32 = 2;
#[cfg(feature = "subnet-catalog-host")]
pub(crate) const SUBNET_CATALOG_REFRESH_REPORT_SCHEMA_VERSION: u32 = 2;

#[cfg(test)]
mod core_tests;
#[cfg(all(test, feature = "subnet-catalog-host"))]
mod tests;
