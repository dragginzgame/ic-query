use super::cache::cache_request;
use crate::{
    cli::{common::write_text_or_json, help::print_help_or_version},
    nns::{
        NnsCommandError, command_icp_root, now_unix_secs,
        subnet::{commands::refresh_usage, options::CatalogRefreshOptions},
    },
    subnet_catalog::{
        SubnetCatalogRefreshRequest, refresh_subnet_catalog, subnet_catalog_refresh_report_text,
    },
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_catalog_refresh(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    if print_help_or_version(&args, refresh_usage, version_text()) {
        return Ok(());
    }
    let options = CatalogRefreshOptions::parse(args)?;
    let format = options.format;
    let icp_root = command_icp_root()?;
    let request = SubnetCatalogRefreshRequest {
        cache: cache_request(&icp_root, &options.network),
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        lock_stale_after_seconds: options.lock_stale_after_seconds,
        dry_run: options.dry_run,
        output_path: options.output_path,
    };
    let report = refresh_subnet_catalog(&request)?;
    write_text_or_json(format, &report, subnet_catalog_refresh_report_text)
}
