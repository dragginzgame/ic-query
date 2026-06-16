use super::cache::cache_request;
use crate::nns::{
    NnsCommandError, command_args,
    node::{
        commands::node_info_usage,
        options::node_info_options,
        report::{NnsNodeInfoRequest, build_nns_node_info_report, nns_node_info_report_text},
    },
    now_unix_secs, write_text_or_json,
};
use std::ffi::OsString;

pub(super) fn run_node_info(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    let Some(args) = command_args(args, node_info_usage) else {
        return Ok(());
    };
    let options = node_info_options(args)?;
    let request = NnsNodeInfoRequest {
        cache: cache_request(&options.network)?,
        source_endpoint: options.source_endpoint,
        input: options.input,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_node_info_report(&request)?;
    write_text_or_json(options.format, &report, nns_node_info_report_text)
}
