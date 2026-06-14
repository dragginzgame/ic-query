use super::{
    commands::{NODE_SPEC, node_info_usage, node_list_usage, node_refresh_usage},
    options::{node_info_options, node_list_options, node_refresh_options},
};
use crate::{
    cli::help::print_help_or_version,
    nns::{
        NnsCommandError, leaf,
        node::report::{
            NnsNodeCacheRequest, NnsNodeInfoRequest, NnsNodeListRequest, NnsNodeRefreshRequest,
            build_nns_node_info_report, build_nns_node_list_report, nns_node_info_report_text,
            nns_node_list_report_text, nns_node_list_report_verbose_text,
            nns_node_refresh_report_text, refresh_nns_node_report,
        },
        now_unix_secs, write_text_or_json,
    },
    project::icp_root,
    version_text,
};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_leaf(
        args,
        &NODE_SPEC,
        run_node_list,
        run_node_info,
        run_node_refresh,
    )
}

fn run_node_list(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    if print_help_or_version(&args, node_list_usage, version_text()) {
        return Ok(());
    }
    let options = node_list_options(args)?;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
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

fn run_node_info(args: Vec<OsString>) -> Result<(), NnsCommandError> {
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

fn run_node_refresh(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    if print_help_or_version(&args, node_refresh_usage, version_text()) {
        return Ok(());
    }
    let options = node_refresh_options(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
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

fn cache_request(icp_root: &Path, network: &str) -> NnsNodeCacheRequest {
    NnsNodeCacheRequest {
        icp_root: PathBuf::from(icp_root),
        network: network.to_string(),
    }
}
