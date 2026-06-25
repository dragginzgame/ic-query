//! Module: nns::proposals::commands
//!
//! Responsibility: clap specs for NNS proposal list and detail commands.
//! Does not own: option validation, live governance calls, or report rendering.
//! Boundary: defines the public `icq nns proposal` command shape.

use crate::{
    cli::clap::{flag_arg, passthrough_subcommand, render_help, value_arg},
    nns::{
        leaf,
        proposals::{
            report::{
                DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT, NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL,
                NNS_PROPOSAL_SORT_API_LABEL, NNS_PROPOSAL_SORT_ASC_LABEL,
                NNS_PROPOSAL_SORT_DESC_LABEL, NNS_PROPOSAL_STATUS_ANY_LABEL,
                NNS_PROPOSAL_TOPIC_ANY_LABEL,
            },
            values::{
                NNS_PROPOSAL_BALLOTS_FLAG, NNS_PROPOSAL_ID_ARG,
                NNS_PROPOSAL_LIST_REWARD_STATUS_ARG, NNS_PROPOSAL_LIST_SORT_VALUE_NAME,
                NNS_PROPOSAL_VERBOSE_FLAG, NnsProposalListSortArg, NnsProposalRewardStatusArg,
                NnsProposalStatusArg, NnsProposalTopicArg,
            },
        },
    },
};
use clap::Command as ClapCommand;
use clap::builder::{NonEmptyStringValueParser, RangedU64ValueParser};

const NNS_PROPOSAL_LIST_DEFAULT_LIMIT: &str = "25";
const NNS_PROPOSAL_LIST_MAX_LIMIT: u64 = 100;
const NNS_PROPOSAL_REFRESH_DEFAULT_PAGE_SIZE: &str = "100";
const NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE: u64 = 100;

const NNS_PROPOSAL_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns proposal list
  icq nns proposal list --limit 50
  icq nns proposal list --before 132000
  icq nns proposal list --status open
  icq nns proposal list --reward-status settled
  icq nns proposal list --topic governance
  icq nns proposal list --proposer 123456789
  icq nns proposal list --query subnet
  icq nns proposal list --sort reward-status
  icq nns proposal list --sort tally-time
  icq nns proposal list --sort deadline
  icq nns proposal list --sort voting-power
  icq nns proposal list --sort proposed
  icq nns proposal list --sort title --asc
  icq nns proposal list --format json
  icq nns proposal list --source-endpoint https://icp-api.io";

const NNS_PROPOSAL_HELP_AFTER: &str = "\
Examples:
  icq nns proposal list
  icq nns proposal info 132411
  icq nns proposal info 132411 --ballots
  icq nns proposal info 132411 --verbose
  icq nns proposal refresh
  icq nns proposal cache status
  icq nns proposal info 132411 --format json
  icq nns proposal info 132411 --source-endpoint https://icp-api.io";

const NNS_PROPOSAL_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns proposal info 132411
  icq nns proposal info 132411 --ballots
  icq nns proposal info 132411 --verbose
  icq nns proposal info 132411 --format json
  icq nns proposal info 132411 --source-endpoint https://icp-api.io";

const NNS_PROPOSAL_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns proposal refresh
  icq nns proposal refresh --page-size 100
  icq nns proposal refresh --max-pages 5
  icq nns proposal refresh --format json
  icq nns proposal refresh --source-endpoint https://icp-api.io";

const NNS_PROPOSAL_CACHE_HELP_AFTER: &str = "\
Examples:
  icq nns proposal cache list
  icq nns proposal cache status
  icq nns proposal cache status --format json";

const NNS_PROPOSAL_CACHE_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns proposal cache list
  icq nns proposal cache list --format json";

const NNS_PROPOSAL_CACHE_STATUS_HELP_AFTER: &str = "\
Examples:
  icq nns proposal cache status
  icq nns proposal cache status --format json";

fn nns_proposal_list_command_with(
    name: &'static str,
    bin_name: &'static str,
    help_after: &'static str,
) -> ClapCommand {
    ClapCommand::new(name)
        .bin_name(bin_name)
        .about("List NNS governance proposals")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native NNS governance query"),
        )
        .arg(
            value_arg("limit")
                .long("limit")
                .value_name("count")
                .default_value(NNS_PROPOSAL_LIST_DEFAULT_LIMIT)
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..=NNS_PROPOSAL_LIST_MAX_LIMIT))
                .help("Maximum NNS proposals to request from governance"),
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
                .default_value(NNS_PROPOSAL_STATUS_ANY_LABEL)
                .value_parser(clap::value_parser!(NnsProposalStatusArg))
                .help("NNS governance decision status filter"),
        )
        .arg(
            value_arg(NNS_PROPOSAL_LIST_REWARD_STATUS_ARG)
                .long(NNS_PROPOSAL_LIST_REWARD_STATUS_ARG)
                .value_name("any|accept-votes|ready-to-settle|settled|ineligible")
                .default_value(NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL)
                .value_parser(clap::value_parser!(NnsProposalRewardStatusArg))
                .help("NNS governance voting reward status filter"),
        )
        .arg(
            value_arg("topic")
                .long("topic")
                .value_name("topic")
                .default_value(NNS_PROPOSAL_TOPIC_ANY_LABEL)
                .value_parser(clap::value_parser!(NnsProposalTopicArg))
                .help("NNS governance topic filter applied to returned rows"),
        )
        .arg(
            value_arg("proposer")
                .long("proposer")
                .value_name("neuron-id")
                .value_parser(RangedU64ValueParser::<u64>::new().range(1..))
                .help("Filter proposals by exact proposer neuron id"),
        )
        .arg(
            value_arg("query")
                .long("query")
                .value_name("text")
                .value_parser(NonEmptyStringValueParser::new())
                .help("Case-insensitive title, action, summary, or URL text filter applied to view rows"),
        )
        .arg(
            value_arg("sort")
                .long("sort")
                .value_name(NNS_PROPOSAL_LIST_SORT_VALUE_NAME)
                .default_value(NNS_PROPOSAL_SORT_API_LABEL)
                .value_parser(clap::value_parser!(NnsProposalListSortArg))
                .help("Sort proposals locally; status/reward-status/text sorts default ascending, numeric and timestamp sorts default descending"),
        )
        .arg(
            flag_arg(NNS_PROPOSAL_SORT_ASC_LABEL)
                .long(NNS_PROPOSAL_SORT_ASC_LABEL)
                .conflicts_with(NNS_PROPOSAL_SORT_DESC_LABEL)
                .help("Sort ascending for local sort modes; this is the default for status/reward-status/topic/proposer/title/action"),
        )
        .arg(
            flag_arg(NNS_PROPOSAL_SORT_DESC_LABEL)
                .long(NNS_PROPOSAL_SORT_DESC_LABEL)
                .conflicts_with(NNS_PROPOSAL_SORT_ASC_LABEL)
                .help("Sort descending for local sort modes; this is the default for id/tally/tally-time/voting-power/ballots/reject-cost/reward-round/timestamps"),
        )
        .arg(
            flag_arg(NNS_PROPOSAL_VERBOSE_FLAG)
                .long(NNS_PROPOSAL_VERBOSE_FLAG)
                .help("Show per-proposal detail lines in text output"),
        )
        .arg(leaf::network_arg())
        .after_help(help_after)
}

pub(in crate::nns::proposals) fn nns_proposal_list_command() -> ClapCommand {
    nns_proposal_list_command_with(
        "list",
        "icq nns proposal list",
        NNS_PROPOSAL_LIST_HELP_AFTER,
    )
}

fn nns_proposal_detail_command_with(
    name: &'static str,
    bin_name: &'static str,
    help_after: &'static str,
) -> ClapCommand {
    ClapCommand::new(name)
        .bin_name(bin_name)
        .about("Show one NNS governance proposal")
        .disable_help_flag(true)
        .arg(
            value_arg(NNS_PROPOSAL_ID_ARG)
                .value_name(NNS_PROPOSAL_ID_ARG)
                .required(true)
                .value_parser(RangedU64ValueParser::<u64>::new().range(1..))
                .help("NNS governance proposal id"),
        )
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native NNS governance query"),
        )
        .arg(
            flag_arg(NNS_PROPOSAL_BALLOTS_FLAG)
                .long(NNS_PROPOSAL_BALLOTS_FLAG)
                .help("Show NNS proposal ballot rows in text output"),
        )
        .arg(
            flag_arg(NNS_PROPOSAL_VERBOSE_FLAG)
                .long(NNS_PROPOSAL_VERBOSE_FLAG)
                .help("Show full NNS proposal detail text"),
        )
        .arg(leaf::network_arg())
        .after_help(help_after)
}

pub(in crate::nns::proposals) fn nns_proposal_command() -> ClapCommand {
    ClapCommand::new("proposal")
        .bin_name("icq nns proposal")
        .about("Inspect NNS governance proposals")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List NNS governance proposals"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("info").about("Show one NNS governance proposal"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("refresh").about(
            "Force-refresh and cache a complete NNS governance proposal snapshot",
        )))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("cache")
                .about("Inspect local complete NNS governance proposal snapshots"),
        ))
        .after_help(NNS_PROPOSAL_HELP_AFTER)
}

pub(in crate::nns::proposals) fn nns_proposal_info_command() -> ClapCommand {
    nns_proposal_detail_command_with(
        "info",
        "icq nns proposal info",
        NNS_PROPOSAL_INFO_HELP_AFTER,
    )
}

pub(in crate::nns::proposals) fn nns_proposal_refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq nns proposal refresh")
        .about("Force-refresh and cache a complete NNS governance proposal snapshot")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native NNS governance query"),
        )
        .arg(
            value_arg("page-size")
                .long("page-size")
                .value_name("count")
                .default_value(NNS_PROPOSAL_REFRESH_DEFAULT_PAGE_SIZE)
                .value_parser(
                    RangedU64ValueParser::<u32>::new()
                        .range(1..=NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE),
                )
                .help("Maximum NNS proposals to request per governance page"),
        )
        .arg(
            value_arg("max-pages")
                .long("max-pages")
                .value_name("count")
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Stop before publishing if this page count is reached before API exhaustion"),
        )
        .arg(leaf::network_arg())
        .after_help(NNS_PROPOSAL_REFRESH_HELP_AFTER)
}

pub(in crate::nns::proposals) fn nns_proposal_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq nns proposal cache")
        .about("Inspect local complete NNS governance proposal snapshots")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List local complete NNS proposal snapshots"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("status").about(
            "Show local NNS proposal snapshot and refresh-attempt status",
        )))
        .after_help(NNS_PROPOSAL_CACHE_HELP_AFTER)
}

pub(in crate::nns::proposals) fn nns_proposal_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq nns proposal cache list")
        .about("List local complete NNS proposal snapshots")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(leaf::network_arg())
        .after_help(NNS_PROPOSAL_CACHE_LIST_HELP_AFTER)
}

pub(in crate::nns::proposals) fn nns_proposal_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .bin_name("icq nns proposal cache status")
        .about("Show local NNS proposal snapshot and refresh-attempt status")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(leaf::network_arg())
        .after_help(NNS_PROPOSAL_CACHE_STATUS_HELP_AFTER)
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposal_list_usage() -> String {
    render_help(nns_proposal_list_command())
}

pub(in crate::nns::proposals) fn nns_proposal_list_usage_for_error() -> String {
    render_help(nns_proposal_list_command())
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposal_usage() -> String {
    render_help(nns_proposal_command())
}

pub(in crate::nns::proposals) fn nns_proposal_usage_for_error() -> String {
    render_help(nns_proposal_command())
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposal_info_usage() -> String {
    render_help(nns_proposal_info_command())
}

pub(in crate::nns::proposals) fn nns_proposal_info_usage_for_error() -> String {
    render_help(nns_proposal_info_command())
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposal_refresh_usage() -> String {
    render_help(nns_proposal_refresh_command())
}

pub(in crate::nns::proposals) fn nns_proposal_refresh_usage_for_error() -> String {
    render_help(nns_proposal_refresh_command())
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposal_cache_usage() -> String {
    render_help(nns_proposal_cache_command())
}

pub(in crate::nns::proposals) fn nns_proposal_cache_usage_for_error() -> String {
    render_help(nns_proposal_cache_command())
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposal_cache_list_usage() -> String {
    render_help(nns_proposal_cache_list_command())
}

pub(in crate::nns::proposals) fn nns_proposal_cache_list_usage_for_error() -> String {
    render_help(nns_proposal_cache_list_command())
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposal_cache_status_usage() -> String {
    render_help(nns_proposal_cache_status_command())
}

pub(in crate::nns::proposals) fn nns_proposal_cache_status_usage_for_error() -> String {
    render_help(nns_proposal_cache_status_command())
}
