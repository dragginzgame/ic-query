//! Module: sns::commands::run::neurons::refresh
//!
//! Responsibility: run explicit SNS neuron complete-snapshot refresh commands.
//! Does not own: refresh paging, attempt files, cache publishing, or rendering.
//! Boundary: maps refresh CLI options into the SNS neuron refresh request.

use crate::{
    cli::common::write_text_or_json,
    progress::StderrQueryProgress,
    sns::commands::{
        SnsCommandError, options::SnsNeuronsRefreshOptions,
        run::common::cached_lookup_command_parts,
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsNeuronsRefreshRequest, refresh_sns_neurons_cache_with_progress,
    sns_neurons_refresh_report_text,
};
pub(super) fn run_sns_neuron_refresh(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    let options = SnsNeuronsRefreshOptions::from_matches(matches, network);
    let parts = cached_lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsNeuronsRefreshRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        cache_root: parts.cache_root,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let mut progress = StderrQueryProgress::new();
    let report = refresh_sns_neurons_cache_with_progress(&request, &mut progress)?;
    write_text_or_json(format, &report, sns_neurons_refresh_report_text)
}
