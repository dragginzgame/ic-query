use super::super::{
    commands::{command, info_usage, list_usage, refresh_usage, usage},
    model::{
        NnsLeafCacheRequest, NnsLeafCommandSpec, NnsLeafInfoRequest, NnsLeafListRequest,
        NnsLeafRefreshRequest, NnsLeafReports,
    },
    options::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions},
};
use crate::{
    cli::common::write_text_or_json_verbose,
    nns::{
        NnsCommandError, command_args, command_icp_root, now_unix_secs,
        parse_nns_required_subcommand, write_text_or_json,
    },
};
use std::ffi::OsString;

pub(in crate::nns) fn run_cached_leaf<I, Reports>(
    args: I,
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    reports: Reports,
) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
    Reports: NnsLeafReports,
{
    let Some(args) = command_args(args, || usage(spec)) else {
        return Ok(());
    };
    let (command_name, args) = parse_nns_required_subcommand(command(spec), args, || usage(spec))?;

    match command_name.as_str() {
        "list" => run_cached_leaf_list(args, spec, default_source_endpoint, &reports),
        "info" => run_cached_leaf_info(args, spec, default_source_endpoint, &reports),
        "refresh" => run_cached_leaf_refresh(args, spec, default_source_endpoint, &reports),
        _ => unreachable!("nns leaf dispatch command only defines known commands"),
    }
}

struct LeafRuntimeParts<Cache> {
    cache: Cache,
    now_unix_secs: u64,
}

fn leaf_runtime_parts<Cache>(network: &str) -> Result<LeafRuntimeParts<Cache>, NnsCommandError>
where
    Cache: NnsLeafCacheRequest,
{
    let icp_root = command_icp_root()?;
    Ok(LeafRuntimeParts {
        cache: Cache::from_root_network(&icp_root, network),
        now_unix_secs: now_unix_secs()?,
    })
}

fn run_cached_leaf_list<Reports>(
    args: Vec<OsString>,
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let Some(args) = command_args(args, || list_usage(spec, default_source_endpoint)) else {
        return Ok(());
    };
    let options = NnsLeafListOptions::parse(args, spec, default_source_endpoint)?;
    let parts = leaf_runtime_parts::<Reports::Cache>(&options.network)?;
    let request = <Reports::ListRequest as NnsLeafListRequest>::from_leaf_parts(
        parts.cache,
        options.source_endpoint,
        parts.now_unix_secs,
    );
    let report = reports.build_list_report(&request).map_err(Into::into)?;
    write_text_or_json_verbose(
        options.format,
        &report,
        options.verbose,
        |report| reports.list_report_text(report),
        |report| reports.list_report_verbose_text(report),
    )
}

fn run_cached_leaf_info<Reports>(
    args: Vec<OsString>,
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let Some(args) = command_args(args, || info_usage(spec, default_source_endpoint)) else {
        return Ok(());
    };
    let options = NnsLeafInfoOptions::parse(args, spec, default_source_endpoint)?;
    let parts = leaf_runtime_parts::<Reports::Cache>(&options.network)?;
    let request = <Reports::InfoRequest as NnsLeafInfoRequest>::from_leaf_parts(
        parts.cache,
        options.source_endpoint,
        options.input,
        parts.now_unix_secs,
    );
    let report = reports.build_info_report(&request).map_err(Into::into)?;
    write_text_or_json(options.format, &report, |report| {
        reports.info_report_text(report)
    })
}

fn run_cached_leaf_refresh<Reports>(
    args: Vec<OsString>,
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    reports: &Reports,
) -> Result<(), NnsCommandError>
where
    Reports: NnsLeafReports,
{
    let Some(args) = command_args(args, || refresh_usage(spec, default_source_endpoint)) else {
        return Ok(());
    };
    let options = NnsLeafRefreshOptions::parse(args, spec, default_source_endpoint)?;
    let format = options.format;
    let parts = leaf_runtime_parts::<Reports::Cache>(&options.network)?;
    let request = <Reports::RefreshRequest as NnsLeafRefreshRequest>::from_leaf_parts(
        parts.cache,
        options.source_endpoint,
        parts.now_unix_secs,
        options.lock_stale_after_seconds,
        options.dry_run,
        options.output_path,
    );
    let report = reports.refresh_report(&request).map_err(Into::into)?;
    write_text_or_json(format, &report, |report| {
        reports.refresh_report_text(report)
    })
}
