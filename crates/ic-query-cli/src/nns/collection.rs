//! Shared CLI orchestration for NNS Governance collection refresh and status.

use super::{NnsCommandError, leaf::NnsPagedRefreshOptions};
use crate::{
    cli::common::{NetworkOutputOptions, current_unix_secs, write_text_or_json},
    progress::StderrQueryProgress,
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::{
    QueryProgress,
    nns::{
        NnsGovernanceCacheRequest, NnsGovernanceRefreshRequest,
        governance::{
            NnsGovernanceCacheStatusReport, NnsGovernanceRefreshReport,
            nns_governance_cache_status_report_text, nns_governance_refresh_report_text,
        },
    },
};

pub(super) async fn run_refresh<Error>(
    matches: &ArgMatches,
    network: &str,
    refresh: impl AsyncFn(
        &NnsGovernanceRefreshRequest,
        &mut dyn QueryProgress,
    ) -> Result<NnsGovernanceRefreshReport, Error>,
) -> Result<(), NnsCommandError>
where
    Error: Into<NnsCommandError>,
{
    let options = NnsPagedRefreshOptions::from_matches(matches, network);
    let format = options.common.format;
    let request = options.into_request(cache_root()?, current_unix_secs()?);
    let report = refresh(&request, &mut StderrQueryProgress::new())
        .await
        .map_err(Into::into)?;
    write_text_or_json(format, &report, nns_governance_refresh_report_text)
}

pub(super) fn run_cache_status<Error>(
    matches: &ArgMatches,
    network: &str,
    build: fn(&NnsGovernanceCacheRequest) -> Result<NnsGovernanceCacheStatusReport, Error>,
) -> Result<(), NnsCommandError>
where
    Error: Into<NnsCommandError>,
{
    let options = NetworkOutputOptions::from_matches(matches, network);
    let request = NnsGovernanceCacheRequest::new(cache_root()?, options.network);
    let report = build(&request).map_err(Into::into)?;
    write_text_or_json(
        options.format,
        &report,
        nns_governance_cache_status_report_text,
    )
}
