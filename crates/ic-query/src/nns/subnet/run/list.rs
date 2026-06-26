use super::cache_request;
use crate::{
    cli::common::write_text_or_json_verbose,
    nns::{
        NnsCommandError, command_args, now_unix_secs,
        subnet::{commands::list_usage, options::CatalogListOptions},
    },
    subnet_catalog::{
        DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogListRequest, build_subnet_catalog_list_report,
        subnet_catalog_list_report_text, subnet_catalog_list_report_verbose_text,
    },
};
use std::ffi::OsString;

pub(super) fn run_catalog_list(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    let Some(args) = command_args(args, list_usage) else {
        return Ok(());
    };
    let options = CatalogListOptions::parse(args)?;
    let format = options.format;
    let verbose = options.verbose;
    let request = SubnetCatalogListRequest::new(
        cache_request(&options.network)?,
        options.source_endpoint,
        now_unix_secs()?,
        DEFAULT_STALE_AFTER_SECONDS,
    )
    .with_filters(options.filters)
    .with_show_ranges(options.show_ranges)
    .with_range_limit(options.range_limit)
    .with_range_offset(options.range_offset);
    let report = build_subnet_catalog_list_report(&request)?;
    write_text_or_json_verbose(
        format,
        &report,
        verbose,
        subnet_catalog_list_report_text,
        subnet_catalog_list_report_verbose_text,
    )
}
