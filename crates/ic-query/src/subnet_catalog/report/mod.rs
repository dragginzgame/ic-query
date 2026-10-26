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
