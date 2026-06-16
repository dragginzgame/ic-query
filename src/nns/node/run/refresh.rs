use super::cache::cache_request;
use crate::{
    cli::help::print_help_or_version,
    nns::{
        NnsCommandError, command_icp_root,
        node::{
            commands::node_refresh_usage,
            options::node_refresh_options,
            report::{
                NnsNodeRefreshRequest, nns_node_refresh_report_text, refresh_nns_node_report,
            },
        },
        now_unix_secs, write_text_or_json,
    },
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_node_refresh(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    if print_help_or_version(&args, node_refresh_usage, version_text()) {
        return Ok(());
    }
    let options = node_refresh_options(args)?;
    let format = options.format;
    let icp_root = command_icp_root()?;
    let request = NnsNodeRefreshRequest {
        cache: cache_request(&icp_root, &options.network),
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        lock_stale_after_seconds: options.lock_stale_after_seconds,
        dry_run: options.dry_run,
        output_path: options.output_path,
    };
    let report = refresh_nns_node_report(&request)?;
    write_text_or_json(format, &report, nns_node_refresh_report_text)
}
