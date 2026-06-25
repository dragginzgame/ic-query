use super::cache_request;
use crate::{
    cli::common::write_text_or_json,
    nns::{
        NnsCommandError, command_args, now_unix_secs,
        subnet::{commands::info_usage, options::CatalogInfoOptions},
    },
    subnet_catalog::{
        DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogInfoRequest, build_subnet_catalog_info_report,
        subnet_catalog_info_report_text,
    },
};
use std::ffi::OsString;

pub(super) fn run_catalog_info(args: Vec<OsString>) -> Result<(), NnsCommandError> {
    let Some(args) = command_args(args, info_usage) else {
        return Ok(());
    };
    let options = CatalogInfoOptions::parse(args)?;
    let format = options.format;
    let request = SubnetCatalogInfoRequest {
        cache: cache_request(&options.network)?,
        source_endpoint: options.source_endpoint,
        input: options.input,
        forced: options.forced,
        now_unix_secs: now_unix_secs()?,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
    };
    let report = build_subnet_catalog_info_report(&request)?;
    write_text_or_json(format, &report, subnet_catalog_info_report_text)
}
