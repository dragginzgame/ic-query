mod info;
mod list;
mod model;
mod rate;

use crate::{
    QueryProgress,
    subnet_catalog::{
        CatalogLoadOutcome, CatalogReadPolicy, SubnetCatalogCacheRequest, SubnetCatalogHostError,
        SubnetCatalogLoadFailure, SubnetCatalogLoadRequest, SubnetCatalogSource,
        catalog_stale_status, load_subnet_catalog_detailed_with_progress,
    },
};

pub use info::{
    build_subnet_catalog_info_report, build_subnet_catalog_info_report_with_progress,
    build_subnet_catalog_info_report_with_source,
    build_subnet_catalog_info_report_with_source_and_progress,
};
pub use list::{
    build_subnet_catalog_list_report, build_subnet_catalog_list_report_with_progress,
    build_subnet_catalog_list_report_with_source,
    build_subnet_catalog_list_report_with_source_and_progress,
};
pub use model::{
    CatalogStaleStatus, SubnetCatalogFilters, SubnetCatalogInfoReport, SubnetCatalogInfoRequest,
    SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogRefreshReport,
    SubnetCatalogSubnetRow,
};

// Both report shapes project the same admitted catalog and cache provenance.
macro_rules! catalog_report {
    ($report:ident, $cached:ident, $stale:ident, { $($field:ident $(: $value:expr)?,)* }) => {
        $report {
            registry_canister_id: $cached.catalog.provenance().registry_canister_id.clone(),
            registry_version: $cached.catalog.provenance().registry_version,
            assurance: $cached.catalog.provenance().assurance,
            source_endpoints: $cached.catalog.provenance().source_endpoints.clone(),
            agreement_digest: $cached.catalog.provenance().agreement_digest.clone(),
            registry_query_call_count: $cached.catalog.provenance().registry_query_call_count,
            routing_source: $cached.catalog.provenance().routing_source,
            registry_records: $cached.catalog.provenance().registry_records.clone(),
            catalog_digest: $cached.catalog.raw().catalog_digest.clone(),
            cache_disposition: $cached.disposition,
            catalog_schema_version: $cached.catalog.raw().catalog_schema_version,
            catalog_path: $cached.path.display().to_string(),
            fetched_at: $cached.catalog.provenance().fetched_at.clone(),
            catalog_stale: $stale.catalog_stale,
            stale_reason: $stale.stale_reason,
            resolver_backend: $cached.catalog.provenance().resolver_backend.clone(),
            collector_version: $cached.catalog.provenance().collector_version.clone(),
            classification_schema_version: $cached.catalog.provenance().classification_schema_version,
            classification_policy_digest: $cached
                .catalog
                .provenance()
                .classification_policy_digest
                .clone(),
            resolver_schema_version: $cached.catalog.provenance().resolver_schema_version,
            $($field $(: $value)?,)*
        }
    };
}

pub(super) use catalog_report;

async fn load_catalog_for_report(
    cache: &SubnetCatalogCacheRequest,
    policy: &CatalogReadPolicy,
    now_unix_secs: u64,
    stale_after_seconds: u64,
    source: &dyn SubnetCatalogSource,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<(CatalogLoadOutcome, CatalogStaleStatus), SubnetCatalogHostError> {
    let request = SubnetCatalogLoadRequest::cache_only(cache.clone(), now_unix_secs)
        .with_policy(policy.clone());
    let cached = load_subnet_catalog_detailed_with_progress(&request, source, progress)
        .await
        .map_err(SubnetCatalogLoadFailure::into_source)?;
    let stale = catalog_stale_status(cached.catalog.raw(), now_unix_secs, stale_after_seconds);
    Ok((cached, stale))
}
