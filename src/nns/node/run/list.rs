use super::cache_request;
use crate::{
    cli::common::write_text_or_json_verbose,
    nns::{
        NnsCommandError, command_args,
        node::{
            commands::node_list_usage,
            options::node_list_options,
            report::{
                NnsNodeListRequest, build_nns_node_list_report, nns_node_list_report_text,
                nns_node_list_report_verbose_text,
            },
        },
        now_unix_secs,
    },
};
use std::ffi::OsString;

pub(super) fn run_node_list(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    let Some(args) = command_args(args, node_list_usage) else {
        return Ok(());
    };
    let options = node_list_options(args)?;
    let request = NnsNodeListRequest {
        cache: cache_request(&options.network)?,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        filters: options.filters,
    };
    let report = build_nns_node_list_report(&request)?;
    write_text_or_json_verbose(
        options.format,
        &report,
        options.verbose,
        nns_node_list_report_text,
        nns_node_list_report_verbose_text,
    )
}
