use super::cache_request;
use crate::nns::{
    NnsCommandError, command_args,
    node::{commands::node_info_usage, options::node_info_options},
    now_unix_secs, write_text_or_json,
};
use ic_query::nns::node::{
    NnsNodeInfoRequest, build_nns_node_info_report, nns_node_info_report_text,
};
use std::ffi::OsString;

pub(super) fn run_node_info(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    let Some(args) = command_args(args, node_info_usage) else {
        return Ok(());
    };
    let options = node_info_options(args)?;
    let request = NnsNodeInfoRequest::new(
        cache_request(&options.network)?,
        options.source_endpoint,
        options.input,
        now_unix_secs()?,
    );
    let report = build_nns_node_info_report(&request)?;
    write_text_or_json(options.format, &report, nns_node_info_report_text)
}
