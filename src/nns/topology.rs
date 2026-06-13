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
    nns_topology::{
        NnsTopologyCapacityRequest, NnsTopologyCoverageRequest, NnsTopologyGapsRequest,
        NnsTopologyHealthRequest, NnsTopologyProvidersRequest, NnsTopologyRefreshRequest,
        NnsTopologyRegionsRequest, NnsTopologySummaryRequest, NnsTopologyVersionsRequest,
        build_nns_topology_capacity_report, build_nns_topology_coverage_report,
        build_nns_topology_gaps_report, build_nns_topology_health_report,
        build_nns_topology_providers_report, build_nns_topology_regions_report,
        build_nns_topology_summary_report, build_nns_topology_versions_report,
        nns_topology_capacity_report_text, nns_topology_coverage_report_text,
        nns_topology_gaps_report_text, nns_topology_health_report_text,
        nns_topology_providers_report_text, nns_topology_refresh_report_text,
        nns_topology_regions_report_text, nns_topology_summary_report_text,
        nns_topology_versions_report_text, refresh_nns_topology_report,
    },
    version_text,
};
use std::ffi::OsString;

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
const DRY_RUN_ARG: &str = "dry-run";
const LOCK_STALE_AFTER_ARG: &str = "lock-stale-after";

///
/// TopologySummaryOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologySummaryOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
}

///
/// TopologyCoverageOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyCoverageOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
}

///
/// TopologyVersionsOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyVersionsOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
}

///
/// TopologyHealthOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyHealthOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
}

///
/// TopologyGapsOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyGapsOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
}

///
/// TopologyCapacityOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyCapacityOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
}

///
/// TopologyRegionsOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyRegionsOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
}

///
/// TopologyProvidersOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TopologyProvidersOptions {
    pub(super) network: String,
    pub(super) format: super::OutputFormat,
    pub(super) source_endpoint: String,
}

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
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_summary_usage, version_text()) {
        return Ok(());
    }
    let options = TopologySummaryOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologySummaryRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_topology_summary_report(&request)?;
    write_text_or_json(format, &report, nns_topology_summary_report_text)
}

fn run_topology_coverage<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_coverage_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyCoverageOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyCoverageRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_topology_coverage_report(&request)?;
    write_text_or_json(format, &report, nns_topology_coverage_report_text)
}

fn run_topology_versions<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_versions_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyVersionsOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyVersionsRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_topology_versions_report(&request)?;
    write_text_or_json(format, &report, nns_topology_versions_report_text)
}

fn run_topology_health<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_health_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyHealthOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyHealthRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_topology_health_report(&request)?;
    write_text_or_json(format, &report, nns_topology_health_report_text)
}

fn run_topology_gaps<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_gaps_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyGapsOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyGapsRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_topology_gaps_report(&request)?;
    write_text_or_json(format, &report, nns_topology_gaps_report_text)
}

fn run_topology_capacity<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_capacity_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyCapacityOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyCapacityRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_topology_capacity_report(&request)?;
    write_text_or_json(format, &report, nns_topology_capacity_report_text)
}

fn run_topology_regions<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_regions_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyRegionsOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyRegionsRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_topology_regions_report(&request)?;
    write_text_or_json(format, &report, nns_topology_regions_report_text)
}

fn run_topology_providers<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_providers_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyProvidersOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyProvidersRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_topology_providers_report(&request)?;
    write_text_or_json(format, &report, nns_topology_providers_report_text)
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

impl TopologySummaryOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_summary_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_summary_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

impl TopologyCoverageOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_coverage_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_coverage_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

impl TopologyVersionsOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_versions_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_versions_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

impl TopologyHealthOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_health_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_health_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

impl TopologyGapsOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_gaps_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_gaps_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

impl TopologyCapacityOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_capacity_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_capacity_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

impl TopologyRegionsOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_regions_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_regions_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

impl TopologyProvidersOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_providers_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_providers_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
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
    clap::Command::new("topology")
        .bin_name("icq nns topology")
        .about("Inspect joined NNS topology metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            clap::Command::new("summary").about("Summarize cached mainnet NNS topology reports"),
        ))
        .subcommand(passthrough_subcommand(
            clap::Command::new("coverage").about("Show cached mainnet NNS topology join coverage"),
        ))
        .subcommand(passthrough_subcommand(
            clap::Command::new("versions")
                .about("Show cached mainnet NNS topology component registry versions"),
        ))
        .subcommand(passthrough_subcommand(
            clap::Command::new("health").about("Check cached mainnet NNS topology cache health"),
        ))
        .subcommand(passthrough_subcommand(
            clap::Command::new("gaps").about("List cached mainnet NNS topology join gaps"),
        ))
        .subcommand(passthrough_subcommand(
            clap::Command::new("capacity").about("Show cached mainnet NNS node-operator capacity"),
        ))
        .subcommand(passthrough_subcommand(
            clap::Command::new("regions").about("Summarize cached mainnet NNS topology by region"),
        ))
        .subcommand(passthrough_subcommand(
            clap::Command::new("providers")
                .about("Summarize cached mainnet NNS topology by node provider"),
        ))
        .subcommand(passthrough_subcommand(
            clap::Command::new("refresh")
                .about("Refresh cached mainnet NNS topology component reports"),
        ))
}

fn topology_summary_command() -> clap::Command {
    clap::Command::new("summary")
        .bin_name("icq nns topology summary")
        .about("Summarize cached mainnet NNS topology reports")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used if a topology component cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_SUMMARY_HELP_AFTER)
}

fn topology_coverage_command() -> clap::Command {
    clap::Command::new("coverage")
        .bin_name("icq nns topology coverage")
        .about("Show cached mainnet NNS topology join coverage")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used if a topology component cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_COVERAGE_HELP_AFTER)
}

fn topology_versions_command() -> clap::Command {
    clap::Command::new("versions")
        .bin_name("icq nns topology versions")
        .about("Show cached mainnet NNS topology component registry versions")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used if a topology component cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_VERSIONS_HELP_AFTER)
}

fn topology_health_command() -> clap::Command {
    clap::Command::new("health")
        .bin_name("icq nns topology health")
        .about("Check cached mainnet NNS topology cache health")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used if a topology component cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_HEALTH_HELP_AFTER)
}

fn topology_gaps_command() -> clap::Command {
    clap::Command::new("gaps")
        .bin_name("icq nns topology gaps")
        .about("List cached mainnet NNS topology join gaps")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used if a topology component cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_GAPS_HELP_AFTER)
}

fn topology_capacity_command() -> clap::Command {
    clap::Command::new("capacity")
        .bin_name("icq nns topology capacity")
        .about("Show cached mainnet NNS node-operator capacity")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used if the node-operator cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_CAPACITY_HELP_AFTER)
}

fn topology_regions_command() -> clap::Command {
    clap::Command::new("regions")
        .bin_name("icq nns topology regions")
        .about("Summarize cached mainnet NNS topology by region")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used if the data-center cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_REGIONS_HELP_AFTER)
}

fn topology_providers_command() -> clap::Command {
    clap::Command::new("providers")
        .bin_name("icq nns topology providers")
        .about("Summarize cached mainnet NNS topology by node provider")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used if a topology component cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_PROVIDERS_HELP_AFTER)
}

fn topology_refresh_command() -> clap::Command {
    clap::Command::new("refresh")
        .bin_name("icq nns topology refresh")
        .about("Refresh cached mainnet NNS topology component reports")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(crate::nns_node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
                .help("IC API endpoint used for NNS topology component refreshes"),
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
