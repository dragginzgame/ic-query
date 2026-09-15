//! Explicit live two-endpoint acquisition followed by local cache reuse.
use ic_query::subnet_catalog::{
    CatalogAssurance, CatalogReadPolicy, CatalogSourceSelection, LiveSubnetCatalogSource,
    SubnetCatalogCacheRequest, SubnetCatalogLoadRequest, load_subnet_catalog_detailed_with_source,
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args()
        .nth(1)
        .ok_or("supply an empty cache directory")?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let request = SubnetCatalogLoadRequest::refresh_missing_or_invalid(
        SubnetCatalogCacheRequest::new(root, "ic"),
        CatalogSourceSelection::multi_endpoint_agreement(vec![
            "https://ic0.app".into(),
            "https://icp-api.io".into(),
        ]),
        now,
    )
    .with_minimum_assurance(CatalogAssurance::MultiEndpointAgreement);
    let source = LiveSubnetCatalogSource::default();
    let start = Instant::now();
    let cold = load_subnet_catalog_detailed_with_source(&request, &source)?;
    println!(
        "acquisition_ms={} version={} calls={}",
        start.elapsed().as_millis(),
        cold.snapshot_authority().registry_version,
        cold.catalog.provenance().registry_query_call_count
    );
    let start = Instant::now();
    let warm = load_subnet_catalog_detailed_with_source(&request, &source)?;
    println!(
        "reuse_us={} authority_equal={}",
        start.elapsed().as_micros(),
        cold.snapshot_authority() == warm.snapshot_authority()
    );
    let refresh = request.with_policy(CatalogReadPolicy::ForceRefresh {
        source: CatalogSourceSelection::multi_endpoint_agreement(vec![
            "https://ic0.app".into(),
            "https://icp-api.io".into(),
        ]),
    });
    let start = Instant::now();
    let refreshed = load_subnet_catalog_detailed_with_source(&refresh, &source)?;
    println!(
        "refresh_ms={} version={} calls={}",
        start.elapsed().as_millis(),
        refreshed.snapshot_authority().registry_version,
        refreshed.catalog.provenance().registry_query_call_count
    );
    Ok(())
}
