use super::{
    model::NnsLeafReports,
    options::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions},
};
use crate::storage::cache_root;
use crate::{
    cli::common::{current_unix_secs, write_text_or_json_verbose},
    nns::{NnsCommandError, write_text_or_json},
    progress::StderrQueryProgress,
};
use clap::ArgMatches;
use ic_query::nns::{
    NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest,
    NnsInventoryRefreshRequest,
};

pub(in crate::nns) async fn run_cached_leaf<Reports>(
    matches: &ArgMatches,
    network: &str,
    reports: Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    match matches.subcommand() {
        Some(("list", matches)) => run_cached_leaf_list(matches, network, &reports).await,
        Some(("info", matches)) => run_cached_leaf_info(matches, network, &reports).await,
        Some(("refresh", matches)) => run_cached_leaf_refresh(matches, network, &reports).await,
        _ => unreachable!("clap requires a known NNS leaf subcommand"),
    }
}

async fn run_cached_leaf_list<Reports>(
    matches: &ArgMatches,
    network: &str,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let options = NnsLeafListOptions::from_matches(matches, network);
    let cache = NnsInventoryCacheRequest::new(cache_root()?, options.network);
    let request =
        NnsInventoryListRequest::new(cache, options.source_endpoint, current_unix_secs()?);
    let report = reports
        .build_list_report(&request, &mut StderrQueryProgress::new())
        .await
        .map_err(Into::into)?;
    write_text_or_json_verbose(
        options.format,
        &report,
        options.verbose,
        |report| reports.list_report_text(report),
        |report| reports.list_report_verbose_text(report),
    )
}

async fn run_cached_leaf_info<Reports>(
    matches: &ArgMatches,
    network: &str,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let options = NnsLeafInfoOptions::from_matches(matches, network);
    let cache = NnsInventoryCacheRequest::new(cache_root()?, options.network);
    let request = NnsInventoryInfoRequest::new(
        cache,
        options.source_endpoint,
        options.input,
        current_unix_secs()?,
    );
    let report = reports
        .build_info_report(&request, &mut StderrQueryProgress::new())
        .await
        .map_err(Into::into)?;
    write_text_or_json(options.format, &report, |report| {
        reports.info_report_text(report)
    })
}

async fn run_cached_leaf_refresh<Reports>(
    matches: &ArgMatches,
    network: &str,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let options = NnsLeafRefreshOptions::from_matches(matches, network);
    let format = options.format;
    let cache = NnsInventoryCacheRequest::new(cache_root()?, options.network);
    let mut request = NnsInventoryRefreshRequest::new(
        cache,
        options.source_endpoint,
        current_unix_secs()?,
        options.lock_stale_after_seconds,
    )
    .with_dry_run(options.dry_run);
    if let Some(output_path) = options.output_path {
        request = request.with_output_path(output_path);
    }
    let report = reports.refresh_report(&request).await.map_err(Into::into)?;
    write_text_or_json(
        format,
        &report,
        ic_query::nns::nns_inventory_refresh_report_text,
    )
}
