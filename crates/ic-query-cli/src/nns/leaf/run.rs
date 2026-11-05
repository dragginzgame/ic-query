use super::options::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions};
use crate::storage::cache_root;
use crate::{
    cli::common::{current_unix_secs, write_text_or_json_verbose},
    nns::{NnsCommandError, write_text_or_json},
    progress::StderrQueryProgress,
};
use clap::ArgMatches;
use ic_query::QueryProgress;
use ic_query::nns::NnsInventoryCacheRequest;
use ic_query::nns::NnsInventoryInfoRequest;
use ic_query::nns::NnsInventoryListRequest;
use ic_query::nns::NnsInventoryRefreshRequest;
use serde::Serialize;

pub(in crate::nns) async fn run_cached_leaf_list<Report, Error>(
    matches: &ArgMatches,
    network: &str,
    build_report: impl AsyncFn(
        &NnsInventoryListRequest,
        &mut dyn QueryProgress,
    ) -> Result<Report, Error>,
    render: &(dyn Fn(&Report, ic_query_text::TextOptions) -> String + Sync),
) -> Result<(), NnsCommandError>
where
    Report: Serialize,
    Error: Into<NnsCommandError>,
{
    let options = NnsLeafListOptions::from_matches(matches, network);
    let cache = NnsInventoryCacheRequest::new(cache_root()?, options.common.network);
    let request =
        NnsInventoryListRequest::new(cache, options.common.source_endpoint, current_unix_secs()?);
    let report = build_report(&request, &mut StderrQueryProgress::new())
        .await
        .map_err(Into::into)?;
    write_text_or_json_verbose(options.common.format, &report, options.verbose, render)
}

pub(in crate::nns) async fn run_cached_leaf_info<Report, Error>(
    matches: &ArgMatches,
    network: &str,
    build_report: impl AsyncFn(
        &NnsInventoryInfoRequest,
        &mut dyn QueryProgress,
    ) -> Result<Report, Error>,
    render: &(dyn Fn(&Report) -> String + Sync),
) -> Result<(), NnsCommandError>
where
    Report: Serialize,
    Error: Into<NnsCommandError>,
{
    let options = NnsLeafInfoOptions::from_matches(matches, network);
    let cache = NnsInventoryCacheRequest::new(cache_root()?, options.common.network);
    let request = NnsInventoryInfoRequest::new(
        cache,
        options.common.source_endpoint,
        options.input,
        current_unix_secs()?,
    );
    let report = build_report(&request, &mut StderrQueryProgress::new())
        .await
        .map_err(Into::into)?;
    write_text_or_json(options.common.format, &report, render)
}

pub(in crate::nns) async fn run_cached_leaf_refresh<Report, Error>(
    matches: &ArgMatches,
    network: &str,
    refresh_report: impl AsyncFn(&NnsInventoryRefreshRequest) -> Result<Report, Error>,
    render: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    Error: Into<NnsCommandError>,
    Report: Serialize,
{
    let options = NnsLeafRefreshOptions::from_matches(matches, network);
    let format = options.common.format;
    let cache = NnsInventoryCacheRequest::new(cache_root()?, options.common.network);
    let mut request = NnsInventoryRefreshRequest::new(
        cache,
        options.common.source_endpoint,
        current_unix_secs()?,
    );
    request.dry_run = options.dry_run;

    if let Some(output_path) = options.output_path {
        request.output_path = Some(output_path);
    }
    let report = refresh_report(&request).await.map_err(Into::into)?;
    write_text_or_json(format, &report, render)
}
