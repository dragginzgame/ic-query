use super::{NnsCommandError, now_unix_secs, write_text_or_json};
pub(super) use crate::cli::common::{format_arg, source_endpoint_arg};
use crate::duration::parse_duration_seconds;
use crate::project::icp_root;
use crate::subnet_catalog::MAINNET_NETWORK;
use crate::{
    cli::{
        clap::{
            flag_arg, parse_matches, parse_required_subcommand, passthrough_subcommand,
            render_help, required_string, required_typed, typed_option, value_arg,
        },
        common::{FORMAT_ARG, OutputFormat, SOURCE_ENDPOINT_ARG},
        globals::internal_network_arg,
        help::print_help_or_version,
    },
    version_text,
};
use clap::{ArgMatches, Command as ClapCommand};
use serde::Serialize;
use std::{
    ffi::OsString,
    marker::PhantomData,
    path::{Path, PathBuf},
};

const INPUT_ARG: &str = "input";
const NETWORK_ARG: &str = "network";
const LOCK_STALE_AFTER_ARG: &str = "lock-stale-after";
const DRY_RUN_ARG: &str = "dry-run";
const OUTPUT_ARG: &str = "output";
const VERBOSE_ARG: &str = "verbose";
const DEFAULT_LOCK_STALE_AFTER: &str = "30m";

///
/// NnsLeafCommandSpec
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NnsLeafCommandSpec {
    pub(super) command_name: &'static str,
    pub(super) bin_name: &'static str,
    pub(super) about: &'static str,
    pub(super) list_about: &'static str,
    pub(super) info_about: &'static str,
    pub(super) refresh_about: &'static str,
    pub(super) list_help_after: &'static str,
    pub(super) info_help_after: &'static str,
    pub(super) refresh_help_after: &'static str,
    pub(super) input_value_name: &'static str,
    pub(super) input_help: &'static str,
    pub(super) list_source_help: &'static str,
    pub(super) info_source_help: &'static str,
    pub(super) refresh_source_help: &'static str,
    pub(super) verbose_help: &'static str,
    pub(super) dry_run_help: &'static str,
    pub(super) output_help: &'static str,
}

///
/// NnsCommonOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NnsCommonOptions {
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) source_endpoint: String,
}

impl NnsCommonOptions {
    pub(super) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            network: required_string(matches, NETWORK_ARG),
            format: required_typed(matches, FORMAT_ARG),
            source_endpoint: required_string(matches, SOURCE_ENDPOINT_ARG),
        }
    }
}

///
/// NnsLeafListOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NnsLeafListOptions {
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) source_endpoint: String,
    pub(super) verbose: bool,
}

impl NnsLeafListOptions {
    pub(super) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_leaf_matches(list_command(spec, default_source_endpoint), args, || {
                list_usage(spec, default_source_endpoint)
            })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            verbose: matches.get_flag(VERBOSE_ARG),
        })
    }
}

///
/// NnsLeafInfoOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NnsLeafInfoOptions {
    pub(super) input: String,
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) source_endpoint: String,
}

impl NnsLeafInfoOptions {
    pub(super) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_leaf_matches(info_command(spec, default_source_endpoint), args, || {
                info_usage(spec, default_source_endpoint)
            })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            input: required_string(&matches, INPUT_ARG),
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

///
/// NnsLeafRefreshOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NnsLeafRefreshOptions {
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) source_endpoint: String,
    pub(super) lock_stale_after_seconds: u64,
    pub(super) dry_run: bool,
    pub(super) output_path: Option<PathBuf>,
}

pub(super) trait NnsLeafCacheRequest: Clone {
    fn from_root_network(icp_root: &Path, network: &str) -> Self;
}

pub(super) trait NnsLeafListRequest {
    type Cache: NnsLeafCacheRequest;

    fn from_leaf_parts(cache: Self::Cache, source_endpoint: String, now_unix_secs: u64) -> Self;
}

pub(super) trait NnsLeafInfoRequest {
    type Cache: NnsLeafCacheRequest;

    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        input: String,
        now_unix_secs: u64,
    ) -> Self;
}

pub(super) trait NnsLeafRefreshRequest {
    type Cache: NnsLeafCacheRequest;

    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
        dry_run: bool,
        output_path: Option<PathBuf>,
    ) -> Self;
}

pub(super) struct NnsLeafReportFns<
    ListRequest,
    InfoRequest,
    RefreshRequest,
    ListReport,
    InfoReport,
    RefreshReport,
    HostError,
> {
    pub(super) build_list_report: fn(&ListRequest) -> Result<ListReport, HostError>,
    pub(super) build_info_report: fn(&InfoRequest) -> Result<InfoReport, HostError>,
    pub(super) refresh_report: fn(&RefreshRequest) -> Result<RefreshReport, HostError>,
    pub(super) list_report_text: fn(&ListReport) -> String,
    pub(super) list_report_verbose_text: fn(&ListReport) -> String,
    pub(super) info_report_text: fn(&InfoReport) -> String,
    pub(super) refresh_report_text: fn(&RefreshReport) -> String,
    marker: PhantomData<fn() -> HostError>,
}

impl<ListRequest, InfoRequest, RefreshRequest, ListReport, InfoReport, RefreshReport, HostError>
    NnsLeafReportFns<
        ListRequest,
        InfoRequest,
        RefreshRequest,
        ListReport,
        InfoReport,
        RefreshReport,
        HostError,
    >
{
    pub(super) const fn new(
        build_list_report: fn(&ListRequest) -> Result<ListReport, HostError>,
        build_info_report: fn(&InfoRequest) -> Result<InfoReport, HostError>,
        refresh_report: fn(&RefreshRequest) -> Result<RefreshReport, HostError>,
        list_report_text: fn(&ListReport) -> String,
        list_report_verbose_text: fn(&ListReport) -> String,
        info_report_text: fn(&InfoReport) -> String,
        refresh_report_text: fn(&RefreshReport) -> String,
    ) -> Self {
        Self {
            build_list_report,
            build_info_report,
            refresh_report,
            list_report_text,
            list_report_verbose_text,
            info_report_text,
            refresh_report_text,
            marker: PhantomData,
        }
    }
}

impl NnsLeafRefreshOptions {
    pub(super) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_leaf_matches(refresh_command(spec, default_source_endpoint), args, || {
                refresh_usage(spec, default_source_endpoint)
            })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            lock_stale_after_seconds: required_typed(&matches, LOCK_STALE_AFTER_ARG),
            dry_run: matches.get_flag(DRY_RUN_ARG),
            output_path: typed_option(&matches, OUTPUT_ARG),
        })
    }
}

pub(super) fn run_leaf<I>(
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

pub(super) fn run_cached_leaf<
    I,
    Cache,
    ListRequest,
    InfoRequest,
    RefreshRequest,
    ListReport,
    InfoReport,
    RefreshReport,
    HostError,
>(
    args: I,
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    reports: NnsLeafReportFns<
        ListRequest,
        InfoRequest,
        RefreshRequest,
        ListReport,
        InfoReport,
        RefreshReport,
        HostError,
    >,
) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
    Cache: NnsLeafCacheRequest,
    ListRequest: NnsLeafListRequest<Cache = Cache>,
    InfoRequest: NnsLeafInfoRequest<Cache = Cache>,
    RefreshRequest: NnsLeafRefreshRequest<Cache = Cache>,
    ListReport: Serialize,
    InfoReport: Serialize,
    RefreshReport: Serialize,
    HostError: Into<NnsCommandError>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, || usage(spec), version_text()) {
        return Ok(());
    }
    let (command_name, args) = parse_required_subcommand(command(spec), args)
        .map_err(|_| NnsCommandError::Usage(usage(spec)))?;

    match command_name.as_str() {
        "list" => run_cached_leaf_list::<
            Cache,
            ListRequest,
            InfoRequest,
            RefreshRequest,
            ListReport,
            InfoReport,
            RefreshReport,
            HostError,
        >(args, spec, default_source_endpoint, &reports),
        "info" => run_cached_leaf_info::<
            Cache,
            ListRequest,
            InfoRequest,
            RefreshRequest,
            ListReport,
            InfoReport,
            RefreshReport,
            HostError,
        >(args, spec, default_source_endpoint, &reports),
        "refresh" => run_cached_leaf_refresh::<
            Cache,
            ListRequest,
            InfoRequest,
            RefreshRequest,
            ListReport,
            InfoReport,
            RefreshReport,
            HostError,
        >(args, spec, default_source_endpoint, &reports),
        _ => unreachable!("nns leaf dispatch command only defines known commands"),
    }
}

fn run_cached_leaf_list<
    Cache,
    ListRequest,
    InfoRequest,
    RefreshRequest,
    ListReport,
    InfoReport,
    RefreshReport,
    HostError,
>(
    args: Vec<OsString>,
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    reports: &NnsLeafReportFns<
        ListRequest,
        InfoRequest,
        RefreshRequest,
        ListReport,
        InfoReport,
        RefreshReport,
        HostError,
    >,
) -> Result<(), NnsCommandError>
where
    Cache: NnsLeafCacheRequest,
    ListRequest: NnsLeafListRequest<Cache = Cache>,
    ListReport: Serialize,
    HostError: Into<NnsCommandError>,
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
    let request = ListRequest::from_leaf_parts(
        Cache::from_root_network(&icp_root, &options.network),
        options.source_endpoint,
        now_unix_secs()?,
    );
    let report = (reports.build_list_report)(&request).map_err(Into::into)?;
    write_text_or_json(options.format, &report, |report| {
        if options.verbose {
            (reports.list_report_verbose_text)(report)
        } else {
            (reports.list_report_text)(report)
        }
    })
}

fn run_cached_leaf_info<
    Cache,
    ListRequest,
    InfoRequest,
    RefreshRequest,
    ListReport,
    InfoReport,
    RefreshReport,
    HostError,
>(
    args: Vec<OsString>,
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    reports: &NnsLeafReportFns<
        ListRequest,
        InfoRequest,
        RefreshRequest,
        ListReport,
        InfoReport,
        RefreshReport,
        HostError,
    >,
) -> Result<(), NnsCommandError>
where
    Cache: NnsLeafCacheRequest,
    InfoRequest: NnsLeafInfoRequest<Cache = Cache>,
    InfoReport: Serialize,
    HostError: Into<NnsCommandError>,
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
    let request = InfoRequest::from_leaf_parts(
        Cache::from_root_network(&icp_root, &options.network),
        options.source_endpoint,
        options.input,
        now_unix_secs()?,
    );
    let report = (reports.build_info_report)(&request).map_err(Into::into)?;
    write_text_or_json(options.format, &report, reports.info_report_text)
}

fn run_cached_leaf_refresh<
    Cache,
    ListRequest,
    InfoRequest,
    RefreshRequest,
    ListReport,
    InfoReport,
    RefreshReport,
    HostError,
>(
    args: Vec<OsString>,
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    reports: &NnsLeafReportFns<
        ListRequest,
        InfoRequest,
        RefreshRequest,
        ListReport,
        InfoReport,
        RefreshReport,
        HostError,
    >,
) -> Result<(), NnsCommandError>
where
    Cache: NnsLeafCacheRequest,
    RefreshRequest: NnsLeafRefreshRequest<Cache = Cache>,
    RefreshReport: Serialize,
    HostError: Into<NnsCommandError>,
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
    let request = RefreshRequest::from_leaf_parts(
        Cache::from_root_network(&icp_root, &options.network),
        options.source_endpoint,
        now_unix_secs()?,
        options.lock_stale_after_seconds,
        options.dry_run,
        options.output_path,
    );
    let report = (reports.refresh_report)(&request).map_err(Into::into)?;
    write_text_or_json(format, &report, reports.refresh_report_text)
}

fn parse_leaf_matches<I>(
    command: ClapCommand,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<ArgMatches, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    parse_matches(command, args).map_err(|_| NnsCommandError::Usage(usage()))
}

pub(super) fn command(spec: &NnsLeafCommandSpec) -> ClapCommand {
    ClapCommand::new(spec.command_name)
        .bin_name(spec.bin_name)
        .about(spec.about)
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about(spec.list_about),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("info").about(spec.info_about),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("refresh").about(spec.refresh_about),
        ))
}

pub(super) fn list_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    ClapCommand::new("list")
        .bin_name(format!("{} list", spec.bin_name))
        .about(spec.list_about)
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(source_endpoint_arg(default_source_endpoint).help(spec.list_source_help))
        .arg(
            flag_arg(VERBOSE_ARG)
                .long(VERBOSE_ARG)
                .help(spec.verbose_help),
        )
        .arg(network_arg())
        .after_help(spec.list_help_after)
}

pub(super) fn info_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    ClapCommand::new("info")
        .bin_name(format!("{} info", spec.bin_name))
        .about(spec.info_about)
        .disable_help_flag(true)
        .arg(
            value_arg(INPUT_ARG)
                .value_name(spec.input_value_name)
                .required(true)
                .help(spec.input_help),
        )
        .arg(format_arg())
        .arg(source_endpoint_arg(default_source_endpoint).help(spec.info_source_help))
        .arg(network_arg())
        .after_help(spec.info_help_after)
}

pub(super) fn refresh_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name(format!("{} refresh", spec.bin_name))
        .about(spec.refresh_about)
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(source_endpoint_arg(default_source_endpoint).help(spec.refresh_source_help))
        .arg(refresh_lock_stale_after_arg())
        .arg(
            flag_arg(DRY_RUN_ARG)
                .long(DRY_RUN_ARG)
                .help(spec.dry_run_help),
        )
        .arg(output_path_arg().help(spec.output_help))
        .arg(network_arg())
        .after_help(spec.refresh_help_after)
}

pub(super) fn usage(spec: &NnsLeafCommandSpec) -> String {
    render_help(command(spec))
}

pub(super) fn list_usage(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> String {
    render_help(list_command(spec, default_source_endpoint))
}

pub(super) fn info_usage(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> String {
    render_help(info_command(spec, default_source_endpoint))
}

pub(super) fn refresh_usage(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> String {
    render_help(refresh_command(spec, default_source_endpoint))
}

pub(super) fn network_arg() -> clap::Arg {
    internal_network_arg().default_value(MAINNET_NETWORK)
}

pub(super) fn refresh_lock_stale_after_arg() -> clap::Arg {
    value_arg(LOCK_STALE_AFTER_ARG)
        .long(LOCK_STALE_AFTER_ARG)
        .value_name("duration")
        .default_value(DEFAULT_LOCK_STALE_AFTER)
        .value_parser(clap::builder::ValueParser::new(
            parse_refresh_lock_stale_after,
        ))
        .help("Treat an existing refresh lock as stale after this duration; defaults to 30m")
}

pub(super) fn output_path_arg() -> clap::Arg {
    value_arg(OUTPUT_ARG)
        .long(OUTPUT_ARG)
        .value_name("path")
        .value_parser(clap::value_parser!(PathBuf))
}

fn parse_refresh_lock_stale_after(value: &str) -> Result<u64, String> {
    parse_duration_seconds(value).map_err(|err| err.to_string())
}
