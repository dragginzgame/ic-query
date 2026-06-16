use super::cache::cache_request;
use crate::{
    cli::common::write_text_or_json,
    nns::{
        NnsCommandError, command_args, now_unix_secs,
        subnet::{commands::refresh_usage, options::CatalogRefreshOptions},
    },
    subnet_catalog::{
        SubnetCatalogRefreshRequest, refresh_subnet_catalog, subnet_catalog_refresh_report_text,
    },
};
use std::ffi::OsString;

pub(super) fn run_catalog_refresh(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    let Some(args) = command_args(args, refresh_usage) else {
        return Ok(());
    };
    let options = CatalogRefreshOptions::parse(args)?;
    let format = options.format;
    let request = SubnetCatalogRefreshRequest {
        cache: cache_request(&options.network)?,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        lock_stale_after_seconds: options.lock_stale_after_seconds,
        dry_run: options.dry_run,
        output_path: options.output_path,
    };
    let report = refresh_subnet_catalog(&request)?;
    write_text_or_json(format, &report, subnet_catalog_refresh_report_text)
}
