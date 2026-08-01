use super::{
    model::{
        NnsLeafCacheRequest, NnsLeafCommandSpec, NnsLeafInfoRequest, NnsLeafListRequest,
        NnsLeafRefreshRequest, NnsLeafReports,
    },
    options::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions},
};
use crate::{
    cli::common::write_text_or_json_verbose,
    nns::{NnsCommandError, command_cache_root, now_unix_secs, write_text_or_json},
    progress::announce_missing_mainnet_cache,
};
use clap::ArgMatches;

pub(in crate::nns) fn run_cached_leaf<Reports>(
    matches: &ArgMatches,
    network: &str,
    spec: &NnsLeafCommandSpec,
    reports: Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    match matches.subcommand() {
        Some(("list", matches)) => run_cached_leaf_list(matches, network, spec, &reports),
        Some(("info", matches)) => run_cached_leaf_info(matches, network, spec, &reports),
        Some(("refresh", matches)) => run_cached_leaf_refresh(matches, network, &reports),
        _ => unreachable!("clap requires a known NNS leaf subcommand"),
    }
}

struct LeafRuntimeParts<Cache> {
    cache: Cache,
    now_unix_secs: u64,
}

fn leaf_runtime_parts<Cache>(network: &str) -> Result<LeafRuntimeParts<Cache>, NnsCommandError>
where
    Cache: NnsLeafCacheRequest,
{
    let cache_root = command_cache_root()?;
    Ok(LeafRuntimeParts {
        cache: Cache::from_root_network(&cache_root, network),
        now_unix_secs: now_unix_secs()?,
    })
}

fn run_cached_leaf_list<Reports>(
    matches: &ArgMatches,
    network: &str,
    spec: &NnsLeafCommandSpec,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let options = NnsLeafListOptions::from_matches(matches, network);
    let parts = leaf_runtime_parts::<Reports::Cache>(&options.network)?;
    announce_missing_leaf_cache(
        &parts.cache,
        reports,
        spec.command_name,
        &options.source_endpoint,
    );
    let request = <Reports::ListRequest as NnsLeafListRequest>::from_leaf_parts(
        parts.cache,
        options.source_endpoint,
        parts.now_unix_secs,
    );
    let report = reports.build_list_report(&request).map_err(Into::into)?;
    write_text_or_json_verbose(
        options.format,
        &report,
        options.verbose,
        |report| reports.list_report_text(report),
        |report| reports.list_report_verbose_text(report),
    )
}

fn run_cached_leaf_info<Reports>(
    matches: &ArgMatches,
    network: &str,
    spec: &NnsLeafCommandSpec,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let options = NnsLeafInfoOptions::from_matches(matches, network);
    let parts = leaf_runtime_parts::<Reports::Cache>(&options.network)?;
    announce_missing_leaf_cache(
        &parts.cache,
        reports,
        spec.command_name,
        &options.source_endpoint,
    );
    let request = <Reports::InfoRequest as NnsLeafInfoRequest>::from_leaf_parts(
        parts.cache,
        options.source_endpoint,
        options.input,
        parts.now_unix_secs,
    );
    let report = reports.build_info_report(&request).map_err(Into::into)?;
    write_text_or_json(options.format, &report, |report| {
        reports.info_report_text(report)
    })
}

fn run_cached_leaf_refresh<Reports>(
    matches: &ArgMatches,
    network: &str,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let options = NnsLeafRefreshOptions::from_matches(matches, network);
    let format = options.format;
    let parts = leaf_runtime_parts::<Reports::Cache>(&options.network)?;
    let request = <Reports::RefreshRequest as NnsLeafRefreshRequest>::from_leaf_parts(
        parts.cache,
        options.source_endpoint,
        parts.now_unix_secs,
        options.lock_stale_after_seconds,
        options.dry_run,
        options.output_path,
    );
    let report = reports.refresh_report(&request).map_err(Into::into)?;
    write_text_or_json(format, &report, |report| {
        reports.refresh_report_text(report)
    })
}

fn announce_missing_leaf_cache<Reports>(
    cache: &Reports::Cache,
    reports: &Reports,
    component: &str,
    source_endpoint: &str,
) where
    Reports: NnsLeafReports,
{
    let path = reports.cache_path(cache);
    announce_missing_mainnet_cache(cache.network(), component, &path, source_endpoint);
}
