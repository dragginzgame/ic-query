use crate::{
    cli::{
        clap::{
            flag_arg, parse_matches, parse_required_subcommand, passthrough_subcommand,
            render_help, required_string, required_typed, typed_option, value_arg,
        },
        common::{
            OutputFormat, current_unix_secs, format_arg, source_endpoint_arg, write_text_or_json,
        },
        globals::internal_network_arg,
        help::print_help_or_version,
    },
    project::icp_root,
    sns::report::{
        DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsListRequest, SnsListSort, SnsLookupRequest,
        SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusRequest, SnsNeuronsRefreshRequest,
        SnsNeuronsRequest, SnsNeuronsSort, SnsProposalRequest, SnsProposalStatusFilter,
        SnsProposalsRequest, build_sns_info_report, build_sns_list_report,
        build_sns_neurons_cache_list_report, build_sns_neurons_cache_status_report,
        build_sns_neurons_report, build_sns_params_report, build_sns_proposal_report,
        build_sns_proposals_report, build_sns_token_report, refresh_sns_neurons_cache,
        sns_info_report_text, sns_list_report_text, sns_neurons_cache_list_report_text,
        sns_neurons_cache_status_report_text, sns_neurons_refresh_report_text,
        sns_neurons_report_text, sns_params_report_text, sns_proposal_report_text,
        sns_proposals_report_text, sns_token_report_text,
    },
    version_text,
};
use candid::Principal;
use clap::{Command as ClapCommand, ValueEnum, builder::RangedU64ValueParser};
use serde::Serialize;
use std::{ffi::OsString, io};
use thiserror::Error as ThisError;

const SNS_NEURONS_DEFAULT_LIMIT: &str = "25";
const SNS_NEURONS_LIVE_MAX_LIMIT: u32 = 100;
const SNS_PROPOSALS_DEFAULT_LIMIT: &str = "25";
const SNS_PROPOSALS_MAX_LIMIT: u64 = 100;
const SNS_NEURONS_REFRESH_DEFAULT_PAGE_SIZE: &str = "100";
const SNS_NEURONS_REFRESH_MAX_PAGE_SIZE: u64 = 100;

const SNS_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns list
  icq sns list --sort name
  icq sns list --verbose
  icq --network ic sns list --format json
  icq sns list --source-endpoint https://icp-api.io";

const SNS_INFO_HELP_AFTER: &str = "\
Examples:
  icq sns info 1
  icq sns info 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns info 1 --format json";

const SNS_TOKEN_HELP_AFTER: &str = "\
Examples:
  icq sns token 1
  icq sns token 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns token 1 --format json";

const SNS_PARAMS_HELP_AFTER: &str = "\
Examples:
  icq sns params 1
  icq sns params 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns params 1 --format json";

const SNS_PROPOSALS_HELP_AFTER: &str = "\
Examples:
  icq sns proposals 1
  icq sns proposals 1 --status open
  icq sns proposals 1 --before 100 --limit 50
  icq sns proposals 23ten-uaaaa-aaaaq-aabia-cai --verbose
  icq --network ic sns proposals 1 --format json";

const SNS_PROPOSAL_HELP_AFTER: &str = "\
Examples:
  icq sns proposal 1 387
  icq sns proposal 23ten-uaaaa-aaaaq-aabia-cai 387
  icq sns proposal 1 387 --verbose
  icq --network ic sns proposal 1 387 --format json";

const SNS_NEURONS_HELP_AFTER: &str = "\
Examples:
  icq sns neurons 1
  icq sns neurons 23ten-uaaaa-aaaaq-aabia-cai --limit 10
  icq sns neurons 1 --owner zqfso-syaaa-aaaaq-aaafq-cai
  icq sns neurons 1 --verbose
  icq sns neurons refresh 1
  icq sns neurons cache list
  icq sns neurons cache status 1
  icq sns neurons 1 --limit 500 --sort stake
  icq --network ic sns neurons 1 --format json";

const SNS_NEURONS_CACHE_HELP_AFTER: &str = "\
Examples:
  icq sns neurons cache list
  icq sns neurons cache status 1
  icq sns neurons cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neurons cache status 1 --format json";

const SNS_NEURONS_CACHE_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns neurons cache list
  icq sns neurons cache list --format json";

const SNS_NEURONS_CACHE_STATUS_HELP_AFTER: &str = "\
Examples:
  icq sns neurons cache status 1
  icq sns neurons cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neurons cache status 1 --format json";

const SNS_NEURONS_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq sns neurons refresh 1
  icq sns neurons refresh 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neurons refresh 1 --page-size 100
  icq --network ic sns neurons refresh 1 --format json";

#[derive(Debug, ThisError)]
pub enum SnsCommandError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    Host(#[from] SnsHostError),

    #[error("system clock before unix epoch: {0}")]
    Clock(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsListOptions {
    network: String,
    format: OutputFormat,
    source_endpoint: String,
    verbose: bool,
    sort: SnsListSortArg,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsLookupOptions {
    input: String,
    network: String,
    format: OutputFormat,
    source_endpoint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsNeuronsOptions {
    lookup: SnsLookupOptions,
    limit: u32,
    owner_principal_id: Option<String>,
    sort: SnsNeuronsSortArg,
    verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsProposalsOptions {
    lookup: SnsLookupOptions,
    limit: u32,
    before_proposal_id: Option<u64>,
    status: SnsProposalStatusArg,
    verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsProposalOptions {
    lookup: SnsLookupOptions,
    proposal_id: u64,
    verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsNeuronsCacheListOptions {
    network: String,
    format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsNeuronsCacheStatusOptions {
    input: String,
    network: String,
    format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsNeuronsRefreshOptions {
    lookup: SnsLookupOptions,
    page_size: u32,
    max_pages: Option<u32>,
}

pub fn run<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(sns_command(), args)
        .map_err(|_| SnsCommandError::Usage(usage()))?;

    match command.as_str() {
        "list" => run_sns_list(args),
        "info" => run_sns_info(args),
        "token" => run_sns_token(args),
        "params" => run_sns_params(args),
        "proposal" => run_sns_proposal(args),
        "proposals" => run_sns_proposals(args),
        "neurons" => run_sns_neurons(args),
        _ => unreachable!("sns dispatch command only defines known commands"),
    }
}

fn run_sns_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_list_usage, version_text()) {
        return Ok(());
    }
    let options = SnsListOptions::parse(args)?;
    let format = options.format;
    let request = SnsListRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        verbose: options.verbose,
        sort: options.sort.into(),
    };
    let report = build_sns_list_report(&request)?;
    write_text_or_json(format, &report, sns_list_report_text)
}

fn run_sns_info<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_sns_lookup(
        args,
        sns_info_command,
        sns_info_usage,
        build_sns_info_report,
        sns_info_report_text,
    )
}

fn run_sns_token<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_sns_lookup(
        args,
        sns_token_command,
        sns_token_usage,
        build_sns_token_report,
        sns_token_report_text,
    )
}

fn run_sns_params<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    run_sns_lookup(
        args,
        sns_params_command,
        sns_params_usage,
        build_sns_params_report,
        sns_params_report_text,
    )
}

fn run_sns_proposal<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_proposal_usage, version_text()) {
        return Ok(());
    }
    let options = SnsProposalOptions::parse(args)?;
    let format = options.lookup.format;
    let request = SnsProposalRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.lookup.input,
        proposal_id: options.proposal_id,
        verbose: options.verbose,
    };
    let report = build_sns_proposal_report(&request)?;
    write_text_or_json(format, &report, sns_proposal_report_text)
}

fn run_sns_proposals<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_proposals_usage, version_text()) {
        return Ok(());
    }
    let options = SnsProposalsOptions::parse(args)?;
    let format = options.lookup.format;
    let request = SnsProposalsRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.lookup.input,
        limit: options.limit,
        before_proposal_id: options.before_proposal_id,
        status: options.status.into(),
        verbose: options.verbose,
    };
    let report = build_sns_proposals_report(&request)?;
    write_text_or_json(format, &report, sns_proposals_report_text)
}

fn run_sns_lookup<I, Report>(
    args: I,
    command: fn() -> ClapCommand,
    usage: fn() -> String,
    build_report: fn(&SnsLookupRequest) -> Result<Report, SnsHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
    Report: Serialize,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, usage, version_text()) {
        return Ok(());
    }
    let options = SnsLookupOptions::parse(args, command, usage)?;
    let format = options.format;
    let request = SnsLookupRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.input,
    };
    let report = build_report(&request)?;
    write_text_or_json(format, &report, render_text)
}

fn run_sns_neurons<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_usage, version_text()) {
        return Ok(());
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("refresh") {
        return run_sns_neurons_refresh(args.into_iter().skip(1));
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("cache") {
        return run_sns_neurons_cache(args.into_iter().skip(1));
    }
    let options = SnsNeuronsOptions::parse(args)?;
    if options.sort != SnsNeuronsSortArg::Api && options.owner_principal_id.is_some() {
        return Err(SnsCommandError::Usage(
            "`icq sns neurons --sort <id|stake|maturity|created>` reads the complete full-neuron cache and does not support --owner yet; use --sort api for owner-filtered live queries".to_string(),
        ));
    }
    let format = options.lookup.format;
    let icp_root = if SnsNeuronsSort::from(options.sort).uses_cache() {
        Some(icp_root().map_err(|err| SnsCommandError::Usage(err.to_string()))?)
    } else {
        None
    };
    let request = SnsNeuronsRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.lookup.input,
        limit: options.limit,
        owner_principal_id: options.owner_principal_id,
        sort: options.sort.into(),
        icp_root,
        verbose: options.verbose,
    };
    let report = build_sns_neurons_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_report_text)
}

fn run_sns_neurons_cache<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_cache_usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(sns_neurons_cache_command(), args)
        .map_err(|_| SnsCommandError::Usage(sns_neurons_cache_usage()))?;
    match command.as_str() {
        "list" => run_sns_neurons_cache_list(args),
        "status" => run_sns_neurons_cache_status(args),
        _ => unreachable!("sns neurons cache dispatch command only defines known commands"),
    }
}

fn run_sns_neurons_cache_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_cache_list_usage, version_text()) {
        return Ok(());
    }
    let options = SnsNeuronsCacheListOptions::parse(args)?;
    let format = options.format;
    let request = SnsNeuronsCacheListRequest {
        network: options.network,
        icp_root: icp_root().map_err(|err| SnsCommandError::Usage(err.to_string()))?,
    };
    let report = build_sns_neurons_cache_list_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_cache_list_report_text)
}

fn run_sns_neurons_cache_status<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_cache_status_usage, version_text()) {
        return Ok(());
    }
    let options = SnsNeuronsCacheStatusOptions::parse(args)?;
    let format = options.format;
    let request = SnsNeuronsCacheStatusRequest {
        network: options.network,
        icp_root: icp_root().map_err(|err| SnsCommandError::Usage(err.to_string()))?,
        input: options.input,
    };
    let report = build_sns_neurons_cache_status_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_cache_status_report_text)
}

fn run_sns_neurons_refresh<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_refresh_usage, version_text()) {
        return Ok(());
    }
    let options = SnsNeuronsRefreshOptions::parse(args)?;
    let format = options.lookup.format;
    let request = SnsNeuronsRefreshRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.lookup.input,
        icp_root: icp_root().map_err(|err| SnsCommandError::Usage(err.to_string()))?,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let report = refresh_sns_neurons_cache(&request)?;
    write_text_or_json(format, &report, sns_neurons_refresh_report_text)
}

impl SnsListOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_list_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_list_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            verbose: matches.get_flag("verbose"),
            sort: required_typed(&matches, "sort"),
        })
    }
}

impl SnsLookupOptions {
    fn parse<I>(
        args: I,
        command: fn() -> ClapCommand,
        usage: fn() -> String,
    ) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches(command(), args).map_err(|_| SnsCommandError::Usage(usage()))?;
        Ok(Self::from_matches(&matches))
    }

    fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: required_string(matches, "network"),
            format: required_typed(matches, "format"),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

impl SnsNeuronsOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_usage()))?;
        let options = Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            limit: required_typed(&matches, "limit"),
            owner_principal_id: typed_option::<Principal>(&matches, "owner")
                .map(|principal| principal.to_text()),
            sort: required_typed(&matches, "sort"),
            verbose: matches.get_flag("verbose"),
        };
        options.validate()?;
        Ok(options)
    }

    fn validate(&self) -> Result<(), SnsCommandError> {
        if self.sort == SnsNeuronsSortArg::Api && self.limit > SNS_NEURONS_LIVE_MAX_LIMIT {
            return Err(SnsCommandError::Usage(format!(
                "`icq sns neurons --sort api` can request at most {SNS_NEURONS_LIVE_MAX_LIMIT} live neurons at a time; refresh the cache and use `--sort <id|stake|maturity|created>` for larger limits"
            )));
        }
        Ok(())
    }
}

impl SnsProposalsOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_proposals_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_proposals_usage()))?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            limit: required_typed(&matches, "limit"),
            before_proposal_id: typed_option::<u64>(&matches, "before"),
            status: required_typed(&matches, "status"),
            verbose: matches.get_flag("verbose"),
        })
    }
}

impl SnsProposalOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_proposal_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_proposal_usage()))?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            proposal_id: required_typed(&matches, "proposal-id"),
            verbose: matches.get_flag("verbose"),
        })
    }
}

impl SnsNeuronsCacheListOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_cache_list_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_cache_list_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
        })
    }
}

impl SnsNeuronsCacheStatusOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_cache_status_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_cache_status_usage()))?;
        Ok(Self {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
        })
    }
}

impl SnsNeuronsRefreshOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_refresh_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_refresh_usage()))?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            page_size: required_typed(&matches, "page-size"),
            max_pages: typed_option::<u32>(&matches, "max-pages"),
        })
    }
}

fn sns_command() -> ClapCommand {
    ClapCommand::new("sns")
        .bin_name("icq sns")
        .about("Inspect SNS metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List deployed mainnet SNS instances"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("info").about("Resolve a deployed SNS by list id or root principal"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("token").about(
            "Show SNS ledger token metadata by list id or root principal",
        )))
        .subcommand(passthrough_subcommand(ClapCommand::new("params").about(
            "Show SNS governance nervous system parameters by list id or root principal",
        )))
        .subcommand(passthrough_subcommand(ClapCommand::new("proposal").about(
            "Show one SNS governance proposal by SNS list id or root principal",
        )))
        .subcommand(passthrough_subcommand(ClapCommand::new("proposals").about(
            "List SNS governance proposals by list id or root principal",
        )))
        .subcommand(passthrough_subcommand(ClapCommand::new("neurons").about(
            "List and refresh SNS governance neurons by SNS list id or root principal",
        )))
}

fn sns_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns list")
        .about("List deployed mainnet SNS instances")
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance metadata queries"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full canister IDs in text output"),
        )
        .arg(sort_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_LIST_HELP_AFTER)
}

fn sns_info_command() -> ClapCommand {
    sns_lookup_command(
        "info",
        "icq sns info",
        "Resolve a deployed SNS by list id or root principal",
        "IC API endpoint used for SNS-W and governance metadata queries",
        SNS_INFO_HELP_AFTER,
    )
}

fn sns_token_command() -> ClapCommand {
    sns_lookup_command(
        "token",
        "icq sns token",
        "Show SNS ledger token metadata by list id or root principal",
        "IC API endpoint used for SNS-W, governance, and ledger queries",
        SNS_TOKEN_HELP_AFTER,
    )
}

fn sns_params_command() -> ClapCommand {
    sns_lookup_command(
        "params",
        "icq sns params",
        "Show SNS governance nervous system parameters by list id or root principal",
        "IC API endpoint used for SNS-W and governance queries",
        SNS_PARAMS_HELP_AFTER,
    )
}

fn sns_proposal_command() -> ClapCommand {
    ClapCommand::new("proposal")
        .bin_name("icq sns proposal")
        .about("Show one SNS governance proposal by SNS list id or root principal")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(
            value_arg("proposal-id")
                .value_name("proposal-id")
                .required(true)
                .value_parser(RangedU64ValueParser::<u64>::new().range(1..))
                .help("SNS governance proposal id"),
        )
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance queries"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full proposal summary and payload text rendering"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PROPOSAL_HELP_AFTER)
}

fn sns_proposals_command() -> ClapCommand {
    ClapCommand::new("proposals")
        .bin_name("icq sns proposals")
        .about("List SNS governance proposals by list id or root principal")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance queries"),
        )
        .arg(
            value_arg("limit")
                .long("limit")
                .value_name("count")
                .default_value(SNS_PROPOSALS_DEFAULT_LIMIT)
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..=SNS_PROPOSALS_MAX_LIMIT))
                .help("Maximum proposals to request from SNS governance"),
        )
        .arg(
            value_arg("before")
                .long("before")
                .value_name("proposal-id")
                .value_parser(RangedU64ValueParser::<u64>::new().range(1..))
                .help("Return proposals with ids lower than this proposal id"),
        )
        .arg(
            value_arg("status")
                .long("status")
                .value_name("any|open|rejected|adopted|executed|failed")
                .default_value("any")
                .value_parser(clap::value_parser!(SnsProposalStatusArg))
                .help("Governance decision status filter"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full proposal titles and per-proposal detail lines in text output"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PROPOSALS_HELP_AFTER)
}

fn sns_lookup_command(
    name: &'static str,
    bin_name: &'static str,
    about: &'static str,
    source_endpoint_help: &'static str,
    after_help: &'static str,
) -> ClapCommand {
    ClapCommand::new(name)
        .bin_name(bin_name)
        .about(about)
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT).help(source_endpoint_help))
        .arg(internal_network_arg().default_value("ic"))
        .after_help(after_help)
}

fn sns_neurons_command() -> ClapCommand {
    ClapCommand::new("neurons")
        .bin_name("icq sns neurons")
        .about("List and refresh SNS governance neurons by SNS list id or root principal")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance queries"),
        )
        .arg(
            value_arg("limit")
                .long("limit")
                .value_name("count")
                .default_value(SNS_NEURONS_DEFAULT_LIMIT)
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Maximum rows to show; --sort api can request at most 100 live neurons"),
        )
        .arg(
            value_arg("owner")
                .long("owner")
                .value_name("principal")
                .value_parser(principal_value_parser())
                .help("Filter neurons by controlling principal"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full neuron IDs in text output"),
        )
        .arg(neurons_sort_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_HELP_AFTER)
}

fn sns_neurons_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq sns neurons cache")
        .about("Inspect local complete SNS governance neuron snapshots")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List local complete SNS neuron snapshots"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("status")
                .about("Show local SNS neuron snapshot and refresh-attempt status"),
        ))
        .after_help(SNS_NEURONS_CACHE_HELP_AFTER)
}

fn sns_neurons_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns neurons cache list")
        .about("List local complete SNS neuron snapshots")
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_CACHE_LIST_HELP_AFTER)
}

fn sns_neurons_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .bin_name("icq sns neurons cache status")
        .about("Show local SNS neuron snapshot and refresh-attempt status")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_CACHE_STATUS_HELP_AFTER)
}

fn sns_neurons_refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq sns neurons refresh")
        .about("Force-refresh and cache a complete SNS governance neuron snapshot")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance queries"),
        )
        .arg(
            value_arg("page-size")
                .long("page-size")
                .value_name("count")
                .default_value(SNS_NEURONS_REFRESH_DEFAULT_PAGE_SIZE)
                .value_parser(
                    RangedU64ValueParser::<u32>::new().range(1..=SNS_NEURONS_REFRESH_MAX_PAGE_SIZE),
                )
                .help("Maximum neurons to request per SNS governance page"),
        )
        .arg(
            value_arg("max-pages")
                .long("max-pages")
                .value_name("count")
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Stop before publishing if this page count is reached before API exhaustion"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_REFRESH_HELP_AFTER)
}

fn usage() -> String {
    render_help(sns_command())
}

fn sns_list_usage() -> String {
    render_help(sns_list_command())
}

fn sns_info_usage() -> String {
    render_help(sns_info_command())
}

fn sns_token_usage() -> String {
    render_help(sns_token_command())
}

fn sns_params_usage() -> String {
    render_help(sns_params_command())
}

fn sns_proposal_usage() -> String {
    render_help(sns_proposal_command())
}

fn sns_proposals_usage() -> String {
    render_help(sns_proposals_command())
}

fn sns_neurons_usage() -> String {
    render_help(sns_neurons_command())
}

fn sns_neurons_cache_usage() -> String {
    render_help(sns_neurons_cache_command())
}

fn sns_neurons_cache_list_usage() -> String {
    render_help(sns_neurons_cache_list_command())
}

fn sns_neurons_cache_status_usage() -> String {
    render_help(sns_neurons_cache_status_command())
}

fn sns_neurons_refresh_usage() -> String {
    render_help(sns_neurons_refresh_command())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum SnsListSortArg {
    Id,
    Name,
}

impl From<SnsListSortArg> for SnsListSort {
    fn from(value: SnsListSortArg) -> Self {
        match value {
            SnsListSortArg::Id => Self::Id,
            SnsListSortArg::Name => Self::Name,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum SnsNeuronsSortArg {
    #[default]
    Api,
    Id,
    Stake,
    Maturity,
    Created,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum SnsProposalStatusArg {
    #[default]
    Any,
    Open,
    Rejected,
    Adopted,
    Executed,
    Failed,
}

impl From<SnsProposalStatusArg> for SnsProposalStatusFilter {
    fn from(value: SnsProposalStatusArg) -> Self {
        match value {
            SnsProposalStatusArg::Any => Self::Any,
            SnsProposalStatusArg::Open => Self::Open,
            SnsProposalStatusArg::Rejected => Self::Rejected,
            SnsProposalStatusArg::Adopted => Self::Adopted,
            SnsProposalStatusArg::Executed => Self::Executed,
            SnsProposalStatusArg::Failed => Self::Failed,
        }
    }
}

impl From<SnsNeuronsSortArg> for SnsNeuronsSort {
    fn from(value: SnsNeuronsSortArg) -> Self {
        match value {
            SnsNeuronsSortArg::Api => Self::Api,
            SnsNeuronsSortArg::Id => Self::Id,
            SnsNeuronsSortArg::Stake => Self::Stake,
            SnsNeuronsSortArg::Maturity => Self::Maturity,
            SnsNeuronsSortArg::Created => Self::Created,
        }
    }
}

fn sort_arg() -> clap::Arg {
    value_arg("sort")
        .long("sort")
        .value_name("id|name")
        .default_value("id")
        .value_parser(clap::value_parser!(SnsListSortArg))
        .help("Text/JSON row order; ids follow the SNS-W response order")
}

fn neurons_sort_arg() -> clap::Arg {
    value_arg("sort")
        .long("sort")
        .value_name("api|id|stake|maturity|created")
        .default_value("api")
        .value_parser(clap::value_parser!(SnsNeuronsSortArg))
        .help("Row order; api uses a bounded live query, other sorts read the complete cache")
}

fn sns_lookup_input_arg() -> clap::Arg {
    value_arg("input")
        .value_name("id|root-principal")
        .required(true)
        .value_parser(sns_lookup_input_value_parser())
        .help("SNS list id or root canister principal")
}

fn sns_lookup_input_value_parser() -> clap::builder::ValueParser {
    clap::builder::ValueParser::new(|value: &str| {
        if value.parse::<usize>().is_ok_and(|id| id > 0) || Principal::from_text(value).is_ok() {
            Ok(value.to_string())
        } else {
            Err("must be a positive SNS list id or root canister principal".to_string())
        }
    })
}

fn principal_value_parser() -> clap::builder::ValueParser {
    clap::builder::ValueParser::new(|value: &str| {
        Principal::from_text(value).map_err(|err| err.to_string())
    })
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
