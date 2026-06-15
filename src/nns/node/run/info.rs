use super::cache::cache_request;
use crate::{
    cli::help::print_help_or_version,
    nns::{
        NnsCommandError,
        node::{
            commands::node_info_usage,
            options::node_info_options,
            report::{NnsNodeInfoRequest, build_nns_node_info_report, nns_node_info_report_text},
        },
        now_unix_secs, write_text_or_json,
    },
    project::icp_root,
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_node_info(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    if print_help_or_version(&args, node_info_usage, version_text()) {
        return Ok(());
    }
    let options = node_info_options(args)?;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsNodeInfoRequest {
        cache: cache_request(&icp_root, &options.network),
        source_endpoint: options.source_endpoint,
        input: options.input,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_node_info_report(&request)?;
    write_text_or_json(options.format, &report, nns_node_info_report_text)
}
