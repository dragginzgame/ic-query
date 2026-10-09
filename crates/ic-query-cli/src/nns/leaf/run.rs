use super::{
    model::NnsLeafReports,
    options::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions},
};
use crate::{
    cli::common::write_text_or_json_verbose,
    nns::{NnsCommandError, command_cache_root, now_unix_secs, write_text_or_json},
    progress::StderrQueryProgress,
};
use clap::ArgMatches;
use ic_query::nns::{
    NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest,
    NnsInventoryRefreshRequest,
};

pub(in crate::nns) fn run_cached_leaf<Reports>(
    matches: &ArgMatches,
    network: &str,
    reports: Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    match matches.subcommand() {
        Some(("list", matches)) => run_cached_leaf_list(matches, network, &reports),
        Some(("info", matches)) => run_cached_leaf_info(matches, network, &reports),
        Some(("refresh", matches)) => run_cached_leaf_refresh(matches, network, &reports),
        _ => unreachable!("clap requires a known NNS leaf subcommand"),
    }
}

struct LeafRuntimeParts {
    cache: NnsInventoryCacheRequest,
    now_unix_secs: u64,
}

fn leaf_runtime_parts(network: &str) -> Result<LeafRuntimeParts, NnsCommandError> {
    let cache_root = command_cache_root()?;
    Ok(LeafRuntimeParts {
        cache: NnsInventoryCacheRequest::new(cache_root, network),
        now_unix_secs: now_unix_secs()?,
    })
}

fn run_cached_leaf_list<Reports>(
    matches: &ArgMatches,
    network: &str,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let options = NnsLeafListOptions::from_matches(matches, network);
    let parts = leaf_runtime_parts(&options.network)?;
    let request =
        NnsInventoryListRequest::new(parts.cache, options.source_endpoint, parts.now_unix_secs);
    let report = reports
        .build_list_report(&request, &mut StderrQueryProgress::new())
        .map_err(Into::into)?;
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
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let options = NnsLeafInfoOptions::from_matches(matches, network);
    let parts = leaf_runtime_parts(&options.network)?;
    let request = NnsInventoryInfoRequest::new(
        parts.cache,
        options.source_endpoint,
        options.input,
        parts.now_unix_secs,
    );
    let report = reports
        .build_info_report(&request, &mut StderrQueryProgress::new())
        .map_err(Into::into)?;
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
    let parts = leaf_runtime_parts(&options.network)?;
    let mut request = NnsInventoryRefreshRequest::new(
        parts.cache,
        options.source_endpoint,
        parts.now_unix_secs,
        options.lock_stale_after_seconds,
    )
    .with_dry_run(options.dry_run);
    if let Some(output_path) = options.output_path {
        request = request.with_output_path(output_path);
    }
    let report = reports.refresh_report(&request).map_err(Into::into)?;
    write_text_or_json(format, &report, |report| {
        reports.refresh_report_text(report)
    })
}
