//! Module: sns::commands::run::neurons::refresh
//!
//! Responsibility: run explicit SNS neuron complete-snapshot refresh commands.
//! Does not own: refresh paging, attempt files, cache publishing, or rendering.
//! Boundary: maps refresh CLI options into the SNS neuron refresh request.

use crate::sns::commands::options::SnsPagedRefreshOptions;
use crate::{
    cli::common::write_text_or_json,
    progress::StderrQueryProgress,
    sns::commands::{
        SnsCommandError,
        run::common::{command_cache_root, command_unix_secs},
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsNeuronsRefreshRequest, refresh_sns_neurons_cache_with_progress,
    sns_neurons_refresh_report_text,
};
pub(super) async fn run_sns_neuron_refresh(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    let options = SnsPagedRefreshOptions::from_matches(matches, network);
    let lookup = options.lookup;
    let now_unix_secs = command_unix_secs()?;
    let cache_root = command_cache_root()?;
    let format = lookup.format;
    let request = SnsNeuronsRefreshRequest {
        network: lookup.network,
        source_endpoint: lookup.source_endpoint,
        now_unix_secs,
        input: lookup.input,
        cache_root,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let mut progress = StderrQueryProgress::new();
    let report = refresh_sns_neurons_cache_with_progress(&request, &mut progress).await?;
    write_text_or_json(format, &report, sns_neurons_refresh_report_text)
}
