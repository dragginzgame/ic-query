use super::cache::cache_request;
use crate::{
    cli::help::print_help_or_version,
    nns::{
        NnsCommandError, command_icp_root,
        node::{
            commands::node_list_usage,
            options::node_list_options,
            report::{
                NnsNodeListRequest, build_nns_node_list_report, nns_node_list_report_text,
                nns_node_list_report_verbose_text,
            },
        },
        now_unix_secs, write_text_or_json,
    },
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_node_list(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    if print_help_or_version(&args, node_list_usage, version_text()) {
        return Ok(());
    }
    let options = node_list_options(args)?;
    let icp_root = command_icp_root()?;
    let request = NnsNodeListRequest {
        cache: cache_request(&icp_root, &options.network),
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        filters: options.filters,
    };
    let report = build_nns_node_list_report(&request)?;
    write_text_or_json(options.format, &report, |report| {
        if options.verbose {
            nns_node_list_report_verbose_text(report)
        } else {
            nns_node_list_report_text(report)
        }
    })
}
