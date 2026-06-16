use super::cache::cache_request;
use crate::{
    cli::{common::write_text_or_json, help::print_help_or_version},
    nns::{
        NnsCommandError, command_icp_root, now_unix_secs,
        subnet::{commands::list_usage, options::CatalogListOptions},
    },
    subnet_catalog::{
        DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogListRequest, build_subnet_catalog_list_report,
        subnet_catalog_list_report_text, subnet_catalog_list_report_verbose_text,
    },
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_catalog_list(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    if print_help_or_version(&args, list_usage, version_text()) {
        return Ok(());
    }
    let options = CatalogListOptions::parse(args)?;
    let format = options.format;
    let verbose = options.verbose;
    let icp_root = command_icp_root()?;
    let request = SubnetCatalogListRequest {
        cache: cache_request(&icp_root, &options.network),
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
        filters: options.filters,
        show_ranges: options.show_ranges,
        range_limit: options.range_limit,
        range_offset: options.range_offset,
    };
    let report = build_subnet_catalog_list_report(&request)?;
    write_text_or_json(format, &report, |report| {
        if verbose {
            subnet_catalog_list_report_verbose_text(report)
        } else {
            subnet_catalog_list_report_text(report)
        }
    })
}
