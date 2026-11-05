//! Direct Governance inspection for SNS snapshot and refresh-attempt evidence.

use crate::{
    HostCacheError,
    cache_file::open_managed_file,
    sns::report::{
        SNS_CACHE_COMPONENT, SnsCacheStatusReport, SnsCacheStatusRequest, SnsHostError,
        cache_attempt::read_sns_refresh_attempt_status_strict,
        cache_paths::{sns_snapshot_cache_paths, sns_snapshot_network_cache_dir},
        cache_storage::SnsCacheStorageFamily,
        enforce_mainnet_network, load_sns_cache_summary_at,
        target::parse_governance_canister_input,
    },
};

/// Inspect one SNS cache family by Governance principal without discovery or live calls.
pub(in crate::sns::report) fn build_sns_cache_status_report<Family>(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    enforce_mainnet_network(&request.network)?;
    let governance_canister_id = parse_governance_canister_input(&request.input)?;
    let paths = sns_snapshot_cache_paths::<Family>(
        &request.cache_root,
        &request.network,
        &governance_canister_id,
    );
    let mut cache = if let Some(file) = open_managed_file(&request.cache_root, &paths.snapshot_path)
        .map_err(|source| HostCacheError::operation(SNS_CACHE_COMPONENT, source))?
    {
        Some(load_sns_cache_summary_at::<Family>(
            &request.cache_root,
            paths.snapshot_path.clone(),
            &request.network,
            None,
            Some(file),
        )?)
    } else {
        None
    };
    let latest_attempt = read_sns_refresh_attempt_status_strict(
        &request.cache_root,
        &paths.refresh_attempt_path,
        &request.network,
        None,
    )?;
    if let Some(cache) = cache.as_mut() {
        cache.latest_attempt.clone_from(&latest_attempt);
    }
    Ok(SnsCacheStatusReport {
        schema_version: 1,
        network: request.network.clone(),
        cache_root: sns_snapshot_network_cache_dir(&request.cache_root, &request.network)
            .display()
            .to_string(),
        input: request.input.clone(),
        found: cache.is_some(),
        cache,
        expected_cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        latest_attempt,
    })
}
