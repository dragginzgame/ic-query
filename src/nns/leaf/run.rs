use super::{
    commands::{command, info_usage, list_usage, refresh_usage, usage},
    model::{
        NnsLeafCacheRequest, NnsLeafCommandSpec, NnsLeafInfoRequest, NnsLeafListRequest,
        NnsLeafRefreshRequest, NnsLeafReports,
    },
    options::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions},
};
use crate::{
    cli::{clap::parse_required_subcommand, help::print_help_or_version},
    nns::{NnsCommandError, now_unix_secs, write_text_or_json},
    project::icp_root,
    version_text,
};
use std::ffi::OsString;

pub(in crate::nns) fn run_leaf<I>(
    args: I,
    spec: &NnsLeafCommandSpec,
    run_list: fn(Vec<OsString>) -> Result<(), NnsCommandError>,
    run_info: fn(Vec<OsString>) -> Result<(), NnsCommandError>,
    run_refresh: fn(Vec<OsString>) -> Result<(), NnsCommandError>,
) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, || usage(spec), version_text()) {
        return Ok(());
    }
    let (command_name, args) = parse_required_subcommand(command(spec), args)
        .map_err(|_| NnsCommandError::Usage(usage(spec)))?;

    match command_name.as_str() {
        "list" => run_list(args),
        "info" => run_info(args),
        "refresh" => run_refresh(args),
        _ => unreachable!("nns leaf dispatch command only defines known commands"),
    }
}

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
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, || usage(spec), version_text()) {
        return Ok(());
    }
    let (command_name, args) = parse_required_subcommand(command(spec), args)
        .map_err(|_| NnsCommandError::Usage(usage(spec)))?;

    match command_name.as_str() {
        "list" => run_cached_leaf_list(args, spec, default_source_endpoint, &reports),
        "info" => run_cached_leaf_info(args, spec, default_source_endpoint, &reports),
        "refresh" => run_cached_leaf_refresh(args, spec, default_source_endpoint, &reports),
        _ => unreachable!("nns leaf dispatch command only defines known commands"),
    }
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
    if print_help_or_version(
        &args,
        || list_usage(spec, default_source_endpoint),
        version_text(),
    ) {
        return Ok(());
    }
    let options = NnsLeafListOptions::parse(args, spec, default_source_endpoint)?;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = <Reports::ListRequest as NnsLeafListRequest>::from_leaf_parts(
        <Reports::Cache as NnsLeafCacheRequest>::from_root_network(&icp_root, &options.network),
        options.source_endpoint,
        now_unix_secs()?,
    );
    let report = reports.build_list_report(&request).map_err(Into::into)?;
    write_text_or_json(options.format, &report, |report| {
        if options.verbose {
            reports.list_report_verbose_text(report)
        } else {
            reports.list_report_text(report)
        }
    })
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
    if print_help_or_version(
        &args,
        || info_usage(spec, default_source_endpoint),
        version_text(),
    ) {
        return Ok(());
    }
    let options = NnsLeafInfoOptions::parse(args, spec, default_source_endpoint)?;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = <Reports::InfoRequest as NnsLeafInfoRequest>::from_leaf_parts(
        <Reports::Cache as NnsLeafCacheRequest>::from_root_network(&icp_root, &options.network),
        options.source_endpoint,
        options.input,
        now_unix_secs()?,
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
    if print_help_or_version(
        &args,
        || refresh_usage(spec, default_source_endpoint),
        version_text(),
    ) {
        return Ok(());
    }
    let options = NnsLeafRefreshOptions::parse(args, spec, default_source_endpoint)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = <Reports::RefreshRequest as NnsLeafRefreshRequest>::from_leaf_parts(
        <Reports::Cache as NnsLeafCacheRequest>::from_root_network(&icp_root, &options.network),
        options.source_endpoint,
        now_unix_secs()?,
        options.lock_stale_after_seconds,
        options.dry_run,
        options.output_path,
    );
    let report = reports.refresh_report(&request).map_err(Into::into)?;
    write_text_or_json(format, &report, |report| {
        reports.refresh_report_text(report)
    })
}
