use crate::{
    cli::{
        clap::{flag_arg, passthrough_subcommand, render_help, value_arg},
        common::{format_arg, source_endpoint_arg},
        globals::internal_network_arg,
    },
    sns::report::{
        DEFAULT_SNS_SOURCE_ENDPOINT, SnsListSort, SnsNeuronsSort, SnsProposalStatusFilter,
    },
};
use candid::Principal;
use clap::{Command as ClapCommand, ValueEnum, builder::RangedU64ValueParser};

const SNS_NEURONS_DEFAULT_LIMIT: &str = "25";
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

pub(super) fn sns_command() -> ClapCommand {
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

pub(super) fn sns_list_command() -> ClapCommand {
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

pub(super) fn sns_info_command() -> ClapCommand {
    sns_lookup_command(
        "info",
        "icq sns info",
        "Resolve a deployed SNS by list id or root principal",
        "IC API endpoint used for SNS-W and governance metadata queries",
        SNS_INFO_HELP_AFTER,
    )
}

pub(super) fn sns_token_command() -> ClapCommand {
    sns_lookup_command(
        "token",
        "icq sns token",
        "Show SNS ledger token metadata by list id or root principal",
        "IC API endpoint used for SNS-W, governance, and ledger queries",
        SNS_TOKEN_HELP_AFTER,
    )
}

pub(super) fn sns_params_command() -> ClapCommand {
    sns_lookup_command(
        "params",
        "icq sns params",
        "Show SNS governance nervous system parameters by list id or root principal",
        "IC API endpoint used for SNS-W and governance queries",
        SNS_PARAMS_HELP_AFTER,
    )
}

pub(super) fn sns_proposal_command() -> ClapCommand {
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

pub(super) fn sns_proposals_command() -> ClapCommand {
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

pub(super) fn sns_neurons_command() -> ClapCommand {
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

pub(super) fn sns_neurons_cache_command() -> ClapCommand {
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

pub(super) fn sns_neurons_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns neurons cache list")
        .about("List local complete SNS neuron snapshots")
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_CACHE_LIST_HELP_AFTER)
}

pub(super) fn sns_neurons_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .bin_name("icq sns neurons cache status")
        .about("Show local SNS neuron snapshot and refresh-attempt status")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_CACHE_STATUS_HELP_AFTER)
}

pub(super) fn sns_neurons_refresh_command() -> ClapCommand {
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

pub(super) fn usage() -> String {
    render_help(sns_command())
}

pub(super) fn sns_list_usage() -> String {
    render_help(sns_list_command())
}

pub(super) fn sns_info_usage() -> String {
    render_help(sns_info_command())
}

pub(super) fn sns_token_usage() -> String {
    render_help(sns_token_command())
}

pub(super) fn sns_params_usage() -> String {
    render_help(sns_params_command())
}

pub(super) fn sns_proposal_usage() -> String {
    render_help(sns_proposal_command())
}

pub(super) fn sns_proposals_usage() -> String {
    render_help(sns_proposals_command())
}

pub(super) fn sns_neurons_usage() -> String {
    render_help(sns_neurons_command())
}

pub(super) fn sns_neurons_cache_usage() -> String {
    render_help(sns_neurons_cache_command())
}

pub(super) fn sns_neurons_cache_list_usage() -> String {
    render_help(sns_neurons_cache_list_command())
}

pub(super) fn sns_neurons_cache_status_usage() -> String {
    render_help(sns_neurons_cache_status_command())
}

pub(super) fn sns_neurons_refresh_usage() -> String {
    render_help(sns_neurons_refresh_command())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(super) enum SnsListSortArg {
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
pub(super) enum SnsNeuronsSortArg {
    #[default]
    Api,
    Id,
    Stake,
    Maturity,
    Created,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(super) enum SnsProposalStatusArg {
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
