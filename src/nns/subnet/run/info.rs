use super::cache::cache_request;
use crate::{
    cli::{common::write_text_or_json, help::print_help_or_version},
    nns::{
        NnsCommandError, command_icp_root, now_unix_secs,
        subnet::{commands::info_usage, options::CatalogInfoOptions},
    },
    subnet_catalog::{
        DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogInfoRequest, build_subnet_catalog_info_report,
        subnet_catalog_info_report_text,
    },
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_catalog_info(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    if print_help_or_version(&args, info_usage, version_text()) {
        return Ok(());
    }
    let options = CatalogInfoOptions::parse(args)?;
    let format = options.format;
    let icp_root = command_icp_root()?;
    let request = SubnetCatalogInfoRequest {
        cache: cache_request(&icp_root, &options.network),
        source_endpoint: options.source_endpoint,
        input: options.input,
        forced: options.forced,
        now_unix_secs: now_unix_secs()?,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
    };
    let report = build_subnet_catalog_info_report(&request)?;
    write_text_or_json(format, &report, subnet_catalog_info_report_text)
}
