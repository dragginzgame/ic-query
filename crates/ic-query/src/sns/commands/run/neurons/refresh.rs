//! Module: sns::commands::run::neurons::refresh
//!
//! Responsibility: run explicit SNS neuron complete-snapshot refresh commands.
//! Does not own: refresh paging, attempt files, cache publishing, or rendering.
//! Boundary: maps refresh CLI options into the SNS neuron refresh request.

use crate::{
    cli::common::write_text_or_json,
    sns::{
        commands::{
            SnsCommandError,
            options::SnsNeuronsRefreshOptions,
            run::common::{cached_lookup_command_parts, command_args},
            spec::sns_neurons_refresh_usage,
        },
        report::{
            SnsNeuronsRefreshRequest, refresh_sns_neurons_cache, sns_neurons_refresh_report_text,
        },
    },
};
use std::ffi::OsString;

pub(super) fn run_sns_neurons_refresh<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_neurons_refresh_usage) else {
        return Ok(());
    };
    let options = SnsNeuronsRefreshOptions::parse(args)?;
    let parts = cached_lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let request = SnsNeuronsRefreshRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        icp_root: parts.icp_root,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let report = refresh_sns_neurons_cache(&request)?;
    write_text_or_json(format, &report, sns_neurons_refresh_report_text)
}
