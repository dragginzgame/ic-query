pub mod report;

use super::{
    NnsCommandError,
    leaf::{self, NnsCommonOptions},
    now_unix_secs, write_text_or_json,
};
use crate::project::icp_root;
use crate::{
    cli::{
        clap::{
            flag_arg, parse_matches, parse_required_subcommand, passthrough_subcommand,
            render_help, required_typed,
        },
        help::{first_arg_is_help, print_help_or_version},
    },
    nns::topology::report::{
        DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT, NnsTopologyCapacityRequest,
        NnsTopologyCoverageRequest, NnsTopologyGapsRequest, NnsTopologyHealthRequest,
        NnsTopologyProvidersRequest, NnsTopologyRefreshRequest, NnsTopologyRegionsRequest,
        NnsTopologySummaryRequest, NnsTopologyVersionsRequest, build_nns_topology_capacity_report,
        build_nns_topology_coverage_report, build_nns_topology_gaps_report,
        build_nns_topology_health_report, build_nns_topology_providers_report,
        build_nns_topology_regions_report, build_nns_topology_summary_report,
        build_nns_topology_versions_report, nns_topology_capacity_report_text,
        nns_topology_coverage_report_text, nns_topology_gaps_report_text,
        nns_topology_health_report_text, nns_topology_providers_report_text,
        nns_topology_refresh_report_text, nns_topology_regions_report_text,
        nns_topology_summary_report_text, nns_topology_versions_report_text,
        refresh_nns_topology_report,
    },
    version_text,
};
use serde::Serialize;
use std::{ffi::OsString, path::PathBuf};

const TOPOLOGY_SUMMARY_HELP_AFTER: &str = "\
Examples:
  icq nns topology summary
  icq --network ic nns topology summary --format json
  icq nns topology summary --source-endpoint https://icp-api.io";
const TOPOLOGY_COVERAGE_HELP_AFTER: &str = "\
Examples:
  icq nns topology coverage
  icq --network ic nns topology coverage --format json
  icq nns topology coverage --source-endpoint https://icp-api.io";
const TOPOLOGY_VERSIONS_HELP_AFTER: &str = "\
Examples:
  icq nns topology versions
  icq --network ic nns topology versions --format json
  icq nns topology versions --source-endpoint https://icp-api.io";
const TOPOLOGY_HEALTH_HELP_AFTER: &str = "\
Examples:
  icq nns topology health
  icq --network ic nns topology health --format json
  icq nns topology health --source-endpoint https://icp-api.io";
const TOPOLOGY_GAPS_HELP_AFTER: &str = "\
Examples:
  icq nns topology gaps
  icq --network ic nns topology gaps --format json
  icq nns topology gaps --source-endpoint https://icp-api.io";
const TOPOLOGY_CAPACITY_HELP_AFTER: &str = "\
Examples:
  icq nns topology capacity
  icq --network ic nns topology capacity --format json
  icq nns topology capacity --source-endpoint https://icp-api.io";
const TOPOLOGY_REGIONS_HELP_AFTER: &str = "\
Examples:
  icq nns topology regions
  icq --network ic nns topology regions --format json
  icq nns topology regions --source-endpoint https://icp-api.io";
const TOPOLOGY_PROVIDERS_HELP_AFTER: &str = "\
Examples:
  icq nns topology providers
  icq --network ic nns topology providers --format json
  icq nns topology providers --source-endpoint https://icp-api.io";
const TOPOLOGY_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns topology refresh
  icq nns topology refresh --dry-run
  icq --network ic nns topology refresh --format json
  icq nns topology refresh --source-endpoint https://icp-api.io";
const TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if a topology component cache is missing";
const TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if the node-operator cache is missing";
const TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if the data-center cache is missing";
const TOPOLOGY_REFRESH_SOURCE_HELP: &str =
    "IC API endpoint used for NNS topology component refreshes";
const DRY_RUN_ARG: &str = "dry-run";
const LOCK_STALE_AFTER_ARG: &str = "lock-stale-after";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TopologyCommandHelp {
    name: &'static str,
    about: &'static str,
}

const TOPOLOGY_SUBCOMMANDS: &[TopologyCommandHelp] = &[
    TopologyCommandHelp {
        name: "summary",
        about: "Summarize cached mainnet NNS topology reports",
    },
    TopologyCommandHelp {
        name: "coverage",
        about: "Show cached mainnet NNS topology join coverage",
    },
    TopologyCommandHelp {
        name: "versions",
        about: "Show cached mainnet NNS topology component registry versions",
    },
    TopologyCommandHelp {
        name: "health",
        about: "Check cached mainnet NNS topology cache health",
    },
    TopologyCommandHelp {
        name: "gaps",
        about: "List cached mainnet NNS topology join gaps",
    },
    TopologyCommandHelp {
        name: "capacity",
        about: "Show cached mainnet NNS node-operator capacity",
    },
    TopologyCommandHelp {
        name: "regions",
        about: "Summarize cached mainnet NNS topology by region",
    },
    TopologyCommandHelp {
        name: "providers",
        about: "Summarize cached mainnet NNS topology by node provider",
    },
    TopologyCommandHelp {
        name: "refresh",
        about: "Refresh cached mainnet NNS topology component reports",
    },
];

macro_rules! topology_read_options {
    ($name:ident, $request:ident, $command:ident, $usage:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(super) struct $name {
            pub(super) network: String,
            pub(super) format: super::OutputFormat,
            pub(super) source_endpoint: String,
        }

        impl $name {
            pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
            where
                I: IntoIterator<Item = OsString>,
            {
                let matches = parse_matches($command(), args)
                    .map_err(|_| NnsCommandError::Usage($usage()))?;
                let common = NnsCommonOptions::from_matches(&matches);
                Ok(Self {
                    network: common.network,
                    format: common.format,
                    source_endpoint: common.source_endpoint,
                })
            }
        }

        impl TopologyReadOptions<$request> for $name {
            fn parse_args(args: Vec<OsString>) -> Result<Self, NnsCommandError> {
                Self::parse(args)
            }

            fn format(&self) -> super::OutputFormat {
                self.format
            }

            fn into_request(self, icp_root: PathBuf, now_unix_secs: u64) -> $request {
                $request {
                    icp_root,
                    network: self.network,
                    source_endpoint: self.source_endpoint,
                    now_unix_secs,
                }
            }
        }
    };
}

trait TopologyReadOptions<Request>: Sized {
    fn parse_args(args: Vec<OsString>) -> Result<Self, NnsCommandError>;
    fn format(&self) -> super::OutputFormat;
    fn into_request(self, icp_root: PathBuf, now_unix_secs: u64) -> Request;
}

topology_read_options!(
    TopologySummaryOptions,
    NnsTopologySummaryRequest,
    topology_summary_command,
    topology_summary_usage
);
topology_read_options!(
    TopologyCoverageOptions,
    NnsTopologyCoverageRequest,
    topology_coverage_command,
    topology_coverage_usage
);
topology_read_options!(
    TopologyVersionsOptions,
    NnsTopologyVersionsRequest,
    topology_versions_command,
    topology_versions_usage
);
topology_read_options!(
    TopologyHealthOptions,
    NnsTopologyHealthRequest,
    topology_health_command,
    topology_health_usage
);
topology_read_options!(
    TopologyGapsOptions,
    NnsTopologyGapsRequest,
    topology_gaps_command,
    topology_gaps_usage
);
topology_read_options!(
    TopologyCapacityOptions,
    NnsTopologyCapacityRequest,
    topology_capacity_command,
    topology_capacity_usage
);
topology_read_options!(
    TopologyRegionsOptions,
    NnsTopologyRegionsRequest,
    topology_regions_command,
    topology_regions_usage
);
topology_read_options!(
    TopologyProvidersOptions,
    NnsTopologyProvidersRequest,
    topology_providers_command,
    topology_providers_usage
);

///
/// TopologyRefreshOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyRefreshOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
    pub(super) lock_stale_after_seconds: u64,
    pub(super) dry_run: bool,
}

pub(super) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_topology_help_or_version(&args) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(topology_command(), args)
        .map_err(|_| NnsCommandError::Usage(topology_usage()))?;

    match command.as_str() {
        "summary" => run_topology_summary(args),
        "coverage" => run_topology_coverage(args),
        "versions" => run_topology_versions(args),
        "health" => run_topology_health(args),
        "gaps" => run_topology_gaps(args),
        "capacity" => run_topology_capacity(args),
        "regions" => run_topology_regions(args),
        "providers" => run_topology_providers(args),
        "refresh" => run_topology_refresh(args),
        _ => unreachable!("nns topology dispatch command only defines known commands"),
    }
}

fn print_topology_help_or_version(args: &[OsString]) -> bool {
    if first_arg_is_help(args) {
        println!("{}", topology_usage());
        return true;
    }
    if args.first().is_some_and(is_version_flag) {
        println!("{}", version_text());
        return true;
    }
    false
}

fn is_version_flag(arg: &OsString) -> bool {
    arg.to_str()
        .is_some_and(|arg| matches!(arg, "--version" | "-V"))
}

fn run_topology_summary<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_topology_read::<_, TopologySummaryOptions, NnsTopologySummaryRequest, _, _>(
        args,
        topology_summary_usage,
        build_nns_topology_summary_report,
        nns_topology_summary_report_text,
    )
}

fn run_topology_coverage<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_topology_read::<_, TopologyCoverageOptions, NnsTopologyCoverageRequest, _, _>(
        args,
        topology_coverage_usage,
        build_nns_topology_coverage_report,
        nns_topology_coverage_report_text,
    )
}

fn run_topology_versions<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_topology_read::<_, TopologyVersionsOptions, NnsTopologyVersionsRequest, _, _>(
        args,
        topology_versions_usage,
        build_nns_topology_versions_report,
        nns_topology_versions_report_text,
    )
}

fn run_topology_health<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_topology_read::<_, TopologyHealthOptions, NnsTopologyHealthRequest, _, _>(
        args,
        topology_health_usage,
        build_nns_topology_health_report,
        nns_topology_health_report_text,
    )
}

fn run_topology_gaps<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_topology_read::<_, TopologyGapsOptions, NnsTopologyGapsRequest, _, _>(
        args,
        topology_gaps_usage,
        build_nns_topology_gaps_report,
        nns_topology_gaps_report_text,
    )
}

fn run_topology_capacity<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_topology_read::<_, TopologyCapacityOptions, NnsTopologyCapacityRequest, _, _>(
        args,
        topology_capacity_usage,
        build_nns_topology_capacity_report,
        nns_topology_capacity_report_text,
    )
}

fn run_topology_regions<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_topology_read::<_, TopologyRegionsOptions, NnsTopologyRegionsRequest, _, _>(
        args,
        topology_regions_usage,
        build_nns_topology_regions_report,
        nns_topology_regions_report_text,
    )
}

fn run_topology_providers<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_topology_read::<_, TopologyProvidersOptions, NnsTopologyProvidersRequest, _, _>(
        args,
        topology_providers_usage,
        build_nns_topology_providers_report,
        nns_topology_providers_report_text,
    )
}

fn run_topology_read<I, Options, Request, Report, HostError>(
    args: I,
    usage: fn() -> String,
    build_report: fn(&Request) -> Result<Report, HostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
    Options: TopologyReadOptions<Request>,
    Report: Serialize,
    HostError: Into<NnsCommandError>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, usage, version_text()) {
        return Ok(());
    }
    let options = Options::parse_args(args)?;
    let format = options.format();
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = options.into_request(icp_root, now_unix_secs()?);
    let report = build_report(&request).map_err(Into::into)?;
    write_text_or_json(format, &report, render_text)
}

fn run_topology_refresh<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_refresh_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyRefreshOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyRefreshRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        lock_stale_after_seconds: options.lock_stale_after_seconds,
        dry_run: options.dry_run,
    };
    let report = refresh_nns_topology_report(&request)?;
    write_text_or_json(format, &report, nns_topology_refresh_report_text)
}

impl TopologyRefreshOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_refresh_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_refresh_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            lock_stale_after_seconds: required_typed(&matches, LOCK_STALE_AFTER_ARG),
            dry_run: matches.get_flag(DRY_RUN_ARG),
        })
    }
}

pub(super) fn topology_command() -> clap::Command {
    TOPOLOGY_SUBCOMMANDS.iter().fold(
        clap::Command::new("topology")
            .bin_name("icq nns topology")
            .about("Inspect joined NNS topology metadata")
            .disable_help_flag(true),
        |command, subcommand| {
            command.subcommand(passthrough_subcommand(
                clap::Command::new(subcommand.name).about(subcommand.about),
            ))
        },
    )
}

fn topology_summary_command() -> clap::Command {
    topology_read_command(
        "summary",
        "Summarize cached mainnet NNS topology reports",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_SUMMARY_HELP_AFTER,
    )
}

fn topology_coverage_command() -> clap::Command {
    topology_read_command(
        "coverage",
        "Show cached mainnet NNS topology join coverage",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_COVERAGE_HELP_AFTER,
    )
}

fn topology_versions_command() -> clap::Command {
    topology_read_command(
        "versions",
        "Show cached mainnet NNS topology component registry versions",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_VERSIONS_HELP_AFTER,
    )
}

fn topology_health_command() -> clap::Command {
    topology_read_command(
        "health",
        "Check cached mainnet NNS topology cache health",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_HEALTH_HELP_AFTER,
    )
}

fn topology_gaps_command() -> clap::Command {
    topology_read_command(
        "gaps",
        "List cached mainnet NNS topology join gaps",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_GAPS_HELP_AFTER,
    )
}

fn topology_capacity_command() -> clap::Command {
    topology_read_command(
        "capacity",
        "Show cached mainnet NNS node-operator capacity",
        TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP,
        TOPOLOGY_CAPACITY_HELP_AFTER,
    )
}

fn topology_regions_command() -> clap::Command {
    topology_read_command(
        "regions",
        "Summarize cached mainnet NNS topology by region",
        TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP,
        TOPOLOGY_REGIONS_HELP_AFTER,
    )
}

fn topology_providers_command() -> clap::Command {
    topology_read_command(
        "providers",
        "Summarize cached mainnet NNS topology by node provider",
        TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP,
        TOPOLOGY_PROVIDERS_HELP_AFTER,
    )
}

fn topology_refresh_command() -> clap::Command {
    clap::Command::new("refresh")
        .bin_name("icq nns topology refresh")
        .about("Refresh cached mainnet NNS topology component reports")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT)
                .help(TOPOLOGY_REFRESH_SOURCE_HELP),
        )
        .arg(leaf::refresh_lock_stale_after_arg())
        .arg(
            flag_arg(DRY_RUN_ARG)
                .long(DRY_RUN_ARG)
                .help("Fetch and validate without replacing topology component caches"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_REFRESH_HELP_AFTER)
}

fn topology_read_command(
    name: &'static str,
    about: &'static str,
    source_help: &'static str,
    after_help: &'static str,
) -> clap::Command {
    clap::Command::new(name)
        .bin_name(format!("icq nns topology {name}"))
        .about(about)
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(leaf::source_endpoint_arg(DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT).help(source_help))
        .arg(leaf::network_arg())
        .after_help(after_help)
}

pub(super) fn topology_usage() -> String {
    render_help(topology_command())
}

pub(super) fn topology_summary_usage() -> String {
    render_help(topology_summary_command())
}

pub(super) fn topology_coverage_usage() -> String {
    render_help(topology_coverage_command())
}

pub(super) fn topology_versions_usage() -> String {
    render_help(topology_versions_command())
}

pub(super) fn topology_health_usage() -> String {
    render_help(topology_health_command())
}

pub(super) fn topology_gaps_usage() -> String {
    render_help(topology_gaps_command())
}

pub(super) fn topology_capacity_usage() -> String {
    render_help(topology_capacity_command())
}

pub(super) fn topology_regions_usage() -> String {
    render_help(topology_regions_command())
}

pub(super) fn topology_providers_usage() -> String {
    render_help(topology_providers_command())
}

pub(super) fn topology_refresh_usage() -> String {
    render_help(topology_refresh_command())
}
