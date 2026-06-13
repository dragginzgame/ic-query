pub mod report;

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
        DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsInfoRequest, SnsListRequest, SnsListSort,
        SnsNeuronsRefreshRequest, SnsNeuronsRequest, SnsNeuronsSort, SnsParamsRequest,
        SnsTokenRequest, build_sns_info_report, build_sns_list_report, build_sns_neurons_report,
        build_sns_params_report, build_sns_token_report, refresh_sns_neurons_cache,
        sns_info_report_text, sns_list_report_text, sns_neurons_refresh_report_text,
        sns_neurons_report_text, sns_params_report_text, sns_token_report_text,
    },
    version_text,
};
use candid::Principal;
use clap::{Command as ClapCommand, ValueEnum, builder::RangedU64ValueParser};
use std::{ffi::OsString, io};
use thiserror::Error as ThisError;

const SNS_NEURONS_DEFAULT_LIMIT: &str = "25";
const SNS_NEURONS_LIVE_MAX_LIMIT: u32 = 100;
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

const SNS_NEURONS_HELP_AFTER: &str = "\
Examples:
  icq sns neurons 1
  icq sns neurons 23ten-uaaaa-aaaaq-aabia-cai --limit 10
  icq sns neurons 1 --owner zqfso-syaaa-aaaaq-aaafq-cai
  icq sns neurons 1 --verbose
  icq sns neurons refresh 1
  icq sns neurons 1 --limit 500 --sort stake
  icq --network ic sns neurons 1 --format json";

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
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_info_usage, version_text()) {
        return Ok(());
    }
    let options = SnsLookupOptions::parse(args, sns_info_command, sns_info_usage)?;
    let format = options.format;
    let request = SnsInfoRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.input,
    };
    let report = build_sns_info_report(&request)?;
    write_text_or_json(format, &report, sns_info_report_text)
}

fn run_sns_token<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_token_usage, version_text()) {
        return Ok(());
    }
    let options = SnsLookupOptions::parse(args, sns_token_command, sns_token_usage)?;
    let format = options.format;
    let request = SnsTokenRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.input,
    };
    let report = build_sns_token_report(&request)?;
    write_text_or_json(format, &report, sns_token_report_text)
}

fn run_sns_params<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_params_usage, version_text()) {
        return Ok(());
    }
    let options = SnsLookupOptions::parse(args, sns_params_command, sns_params_usage)?;
    let format = options.format;
    let request = SnsParamsRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.input,
    };
    let report = build_sns_params_report(&request)?;
    write_text_or_json(format, &report, sns_params_report_text)
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
        Ok(Self {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
        })
    }
}

impl SnsNeuronsOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_usage()))?;
        let lookup = SnsLookupOptions {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
        };
        let options = Self {
            lookup,
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

impl SnsNeuronsRefreshOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_refresh_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_refresh_usage()))?;
        let lookup = SnsLookupOptions {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
        };
        Ok(Self {
            lookup,
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
    ClapCommand::new("info")
        .bin_name("icq sns info")
        .about("Resolve a deployed SNS by list id or root principal")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance metadata queries"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_INFO_HELP_AFTER)
}

fn sns_token_command() -> ClapCommand {
    ClapCommand::new("token")
        .bin_name("icq sns token")
        .about("Show SNS ledger token metadata by list id or root principal")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W, governance, and ledger queries"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_TOKEN_HELP_AFTER)
}

fn sns_params_command() -> ClapCommand {
    ClapCommand::new("params")
        .bin_name("icq sns params")
        .about("Show SNS governance nervous system parameters by list id or root principal")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance queries"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PARAMS_HELP_AFTER)
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

fn sns_neurons_usage() -> String {
    render_help(sns_neurons_command())
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
mod tests;
