//! Module: sns::commands::run::neurons::refresh
//!
//! Responsibility: run explicit SNS neuron complete-snapshot refresh commands.
//! Does not own: refresh paging, attempt files, cache publishing, or rendering.
//! Boundary: maps refresh CLI options into the SNS neuron refresh request.

use crate::sns::commands::options::SnsPagedRefreshOptions;
use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    progress::StderrQueryProgress,
    sns::commands::SnsCommandError,
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::sns::{refresh_sns_neurons_cache_with_progress, sns_snapshot_refresh_report_text};
pub(super) async fn run_sns_neuron_refresh(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    let options = SnsPagedRefreshOptions::from_matches(matches, network);
    let (format, request) = options.into_request(cache_root()?, current_unix_secs()?);
    let mut progress = StderrQueryProgress::new();
    let report = refresh_sns_neurons_cache_with_progress(&request, &mut progress).await?;
    write_text_or_json(format, &report, sns_snapshot_refresh_report_text)
}
