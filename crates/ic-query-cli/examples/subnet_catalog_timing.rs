//! Explicit live agreement with disk history reuse across fresh sources and processes.
use ic_query::subnet_catalog::{
    CatalogAssurance, CatalogReadPolicy, CatalogSourceSelection, LiveSubnetCatalogSource,
    RegistryHistoryCacheDisposition, SubnetCatalogCacheRequest, SubnetCatalogLoadRequest,
    SubnetCatalogProgressPhase, load_subnet_catalog_detailed_with_source,
};
use std::{
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args()
        .nth(1)
        .ok_or("supply a cache directory; every run performs live agreement")?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let request = SubnetCatalogLoadRequest::refresh_missing_or_invalid(
        SubnetCatalogCacheRequest::new(&root, "ic"),
        CatalogSourceSelection::multi_endpoint_agreement(vec![
            "https://ic0.app".into(),
            "https://icp-api.io".into(),
        ]),
        now,
    )
    .with_minimum_assurance(CatalogAssurance::MultiEndpointAgreement)
    .with_policy(CatalogReadPolicy::ForceRefresh {
        source: CatalogSourceSelection::multi_endpoint_agreement(vec![
            "https://ic0.app".into(),
            "https://icp-api.io".into(),
        ]),
    });
    let source = history_source(Path::new(&root));
    let start = Instant::now();
    let cold = load_subnet_catalog_detailed_with_source(&request, &source)?;
    println!(
        "acquisition_ms={} version={} calls={} agreement_digest={}",
        start.elapsed().as_millis(),
        cold.snapshot_authority().registry_version,
        cold.catalog.provenance().registry_query_call_count,
        cold.catalog
            .provenance()
            .agreement_digest
            .as_deref()
            .unwrap_or("none")
    );
    let start = Instant::now();
    let cache_only = request.clone().with_policy(CatalogReadPolicy::CacheOnly);
    let warm = load_subnet_catalog_detailed_with_source(&cache_only, &source)?;
    println!(
        "reuse_us={} authority_equal={}",
        start.elapsed().as_micros(),
        cold.snapshot_authority() == warm.snapshot_authority()
    );
    drop(source);
    let source = history_source(Path::new(&root));
    let start = Instant::now();
    let refreshed = load_subnet_catalog_detailed_with_source(&request, &source)?;
    println!(
        "fresh_source_refresh_ms={} version={} calls={} agreement_digest={}",
        start.elapsed().as_millis(),
        refreshed.snapshot_authority().registry_version,
        refreshed.catalog.provenance().registry_query_call_count,
        refreshed
            .catalog
            .provenance()
            .agreement_digest
            .as_deref()
            .unwrap_or("none")
    );
    Ok(())
}

fn history_source(root: &Path) -> LiveSubnetCatalogSource {
    LiveSubnetCatalogSource::with_progress(|event| {
        if let SubnetCatalogProgressPhase::HistoryCache { disposition, through_version, reason, .. } = event.phase
            && matches!(disposition, RegistryHistoryCacheDisposition::Reused | RegistryHistoryCacheDisposition::Rejected | RegistryHistoryCacheDisposition::Missing)
        {
            eprintln!("endpoint={} history_cache={disposition:?} through_version={through_version} reason={reason:?}", event.endpoint);
        }
    }).with_history_cache(root)
}
