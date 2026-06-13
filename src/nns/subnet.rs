use super::{NnsCommandError, OutputFormat, leaf, now_unix_secs, write_text_or_json};
use crate::project::icp_root;
use crate::subnet_catalog::{GeographicScope, ResolveAs, SubnetKind, SubnetSpecialization};
use crate::{
    cli::{
        clap::{
            flag_arg, parse_matches, parse_positive_usize, parse_required_subcommand,
            passthrough_subcommand, render_help, required_string, required_typed, typed_option,
            value_arg,
        },
        help::print_help_or_version,
    },
    subnet_catalog::{
        DEFAULT_STALE_AFTER_SECONDS, DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
        SubnetCatalogCacheRequest, SubnetCatalogFilters, SubnetCatalogInfoRequest,
        SubnetCatalogListRequest, SubnetCatalogRefreshRequest, build_subnet_catalog_info_report,
        build_subnet_catalog_list_report, refresh_subnet_catalog, subnet_catalog_info_report_text,
        subnet_catalog_list_report_text, subnet_catalog_list_report_verbose_text,
        subnet_catalog_refresh_report_text,
    },
    version_text,
};
use clap::Command as ClapCommand;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[cfg(test)]
pub(super) const DEFAULT_RANGE_LIMIT: usize = 50;
const DEFAULT_RANGE_LIMIT_ARG: &str = "50";
const INFO_INPUT_VALUE_NAME: &str = "subnet|canister|subnet-prefix";
const INFO_INPUT_HELP: &str = "Subnet/canister principal or unique subnet prefix";
const LIST_HELP_AFTER: &str = "\
Examples:
  icq nns subnet list
  icq nns subnet list --verbose
  icq --network ic nns subnet list --format json
  icq nns subnet list --kind application --specialization fiduciary

Refresh stale cache:
  icq nns subnet refresh";
const INFO_HELP_AFTER: &str = "\
Examples:
  icq nns subnet info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq nns subnet info <subnet-prefix>

Refresh stale cache:
  icq nns subnet refresh";
const REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns subnet refresh
  icq --network ic nns subnet refresh --format json
  icq nns subnet refresh --dry-run --output .ic-query/subnet-catalog/ic/catalog.preview.json";

///
/// CatalogListOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CatalogListOptions {
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) filters: SubnetCatalogFilters,
    pub(super) show_ranges: bool,
    pub(super) verbose: bool,
    pub(super) range_limit: usize,
    pub(super) range_offset: usize,
}

///
/// CatalogInfoOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CatalogInfoOptions {
    pub(super) input: String,
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) forced: Option<ResolveAs>,
}

///
/// CatalogRefreshOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CatalogRefreshOptions {
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) source_endpoint: String,
    pub(super) lock_stale_after_seconds: u64,
    pub(super) dry_run: bool,
    pub(super) output_path: Option<PathBuf>,
}

pub(super) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, subnet_usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(subnet_command(), args)
        .map_err(|_| NnsCommandError::Usage(subnet_usage()))?;

    match command.as_str() {
        "list" => run_catalog_list(args),
        "info" => run_catalog_info(args),
        "refresh" => run_catalog_refresh(args),
        _ => unreachable!("nns subnet dispatch command only defines known commands"),
    }
}

fn run_catalog_list<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, list_usage, version_text()) {
        return Ok(());
    }
    let options = CatalogListOptions::parse(args)?;
    let format = options.format;
    let verbose = options.verbose;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = SubnetCatalogListRequest {
        cache: cache_request(&icp_root, &options.network),
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

fn run_catalog_info<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, info_usage, version_text()) {
        return Ok(());
    }
    let options = CatalogInfoOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = SubnetCatalogInfoRequest {
        cache: cache_request(&icp_root, &options.network),
        input: options.input.clone(),
        forced: options.forced,
        now_unix_secs: now_unix_secs()?,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
    };
    let report = build_subnet_catalog_info_report(&request)?;
    write_text_or_json(format, &report, subnet_catalog_info_report_text)
}

fn run_catalog_refresh<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, refresh_usage, version_text()) {
        return Ok(());
    }
    let options = CatalogRefreshOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
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

impl CatalogListOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(list_command(), args)
            .map_err(|_| NnsCommandError::Usage(list_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            filters: SubnetCatalogFilters {
                kind: typed_option(&matches, "kind"),
                specialization: typed_option(&matches, "specialization"),
                geographic_scope: typed_option(&matches, "geo"),
            },
            show_ranges: matches.get_flag("show-ranges"),
            verbose: matches.get_flag("verbose"),
            range_limit: required_typed(&matches, "range-limit"),
            range_offset: required_typed(&matches, "range-offset"),
        })
    }
}

impl CatalogInfoOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(info_command(), args)
            .map_err(|_| NnsCommandError::Usage(info_usage()))?;
        Ok(Self {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            forced: typed_option(&matches, "as"),
        })
    }
}

impl CatalogRefreshOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(refresh_command(), args)
            .map_err(|_| NnsCommandError::Usage(refresh_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            lock_stale_after_seconds: required_typed(&matches, "lock-stale-after"),
            dry_run: matches.get_flag("dry-run"),
            output_path: typed_option(&matches, "output"),
        })
    }
}

fn cache_request(icp_root: &Path, network: &str) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest {
        icp_root: PathBuf::from(icp_root),
        network: network.to_string(),
    }
}

pub(super) fn subnet_command() -> ClapCommand {
    ClapCommand::new("subnet")
        .bin_name("icq nns subnet")
        .about("Inspect and refresh NNS subnet metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List cached mainnet IC subnets"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("info").about(
            "Resolve a subnet, canister, or subnet prefix to cached subnet info",
        )))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("refresh").about("Force-refresh and cache NNS subnet metadata"),
        ))
}

fn list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq nns subnet list")
        .about("List cached mainnet IC subnets")
        .disable_help_flag(true)
        .arg(
            value_arg("kind")
                .long("kind")
                .value_name("kind")
                .value_parser(clap::value_parser!(SubnetKind))
                .help("Filter by subnet kind: application, cloud_engine, system, or unknown"),
        )
        .arg(
            value_arg("specialization")
                .long("specialization")
                .value_name("specialization")
                .value_parser(clap::value_parser!(SubnetSpecialization))
                .help("Filter by specialization: none, fiduciary, european, or unknown"),
        )
        .arg(
            value_arg("geo")
                .long("geo")
                .value_name("scope")
                .value_parser(clap::value_parser!(GeographicScope))
                .help("Filter by geographic scope: global, europe, or unknown"),
        )
        .arg(leaf::format_arg())
        .arg(
            flag_arg("show-ranges")
                .long("show-ranges")
                .help("Show cached routing ranges after the subnet table"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full subnet principals and catalog metadata in text output"),
        )
        .arg(
            value_arg("range-limit")
                .long("range-limit")
                .value_name("n")
                .default_value(DEFAULT_RANGE_LIMIT_ARG)
                .value_parser(clap::builder::ValueParser::new(parse_positive_usize))
                .help("Maximum routing ranges to show per subnet in text output"),
        )
        .arg(
            value_arg("range-offset")
                .long("range-offset")
                .value_name("n")
                .default_value("0")
                .value_parser(clap::value_parser!(usize))
                .help("Routing range offset for text output"),
        )
        .arg(leaf::network_arg())
        .after_help(LIST_HELP_AFTER)
}

fn info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq nns subnet info")
        .about("Resolve a subnet, canister, or subnet prefix to cached subnet info")
        .disable_help_flag(true)
        .arg(
            value_arg("input")
                .value_name(INFO_INPUT_VALUE_NAME)
                .required(true)
                .help(INFO_INPUT_HELP),
        )
        .arg(
            value_arg("as")
                .long("as")
                .value_name("subnet|canister")
                .value_parser(clap::value_parser!(ResolveAs))
                .help("Force principal interpretation"),
        )
        .arg(leaf::format_arg())
        .arg(leaf::network_arg())
        .after_help(INFO_HELP_AFTER)
}

fn refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq nns subnet refresh")
        .about("Force-refresh and cache NNS subnet metadata")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the NNS registry query"),
        )
        .arg(leaf::refresh_lock_stale_after_arg())
        .arg(
            flag_arg("dry-run")
                .long("dry-run")
                .help("Fetch and validate without replacing the cached catalog"),
        )
        .arg(leaf::output_path_arg().help("Also write the fetched catalog JSON to this path"))
        .arg(leaf::network_arg())
        .after_help(REFRESH_HELP_AFTER)
}

pub(super) fn subnet_usage() -> String {
    render_help(subnet_command())
}

pub(super) fn list_usage() -> String {
    render_help(list_command())
}

pub(super) fn info_usage() -> String {
    render_help(info_command())
}

pub(super) fn refresh_usage() -> String {
    render_help(refresh_command())
}
