//! Module: nns::proposals::commands
//!
//! Responsibility: clap specs for NNS proposal list and detail commands.
//! Does not own: option validation, live governance calls, or report rendering.
//! Boundary: defines the public `icq nns proposal(s)` command shape.

use crate::{
    cli::clap::{flag_arg, render_help, value_arg},
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
                NNS_PROPOSAL_BALLOTS_FLAG, NNS_PROPOSAL_ID_ARG, NNS_PROPOSAL_VERBOSE_FLAG,
                NNS_PROPOSALS_REWARD_STATUS_ARG, NNS_PROPOSALS_SORT_VALUE_NAME,
                NnsProposalRewardStatusArg, NnsProposalStatusArg, NnsProposalTopicArg,
                NnsProposalsSortArg,
            },
        },
    },
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};

const NNS_PROPOSALS_DEFAULT_LIMIT: &str = "25";
const NNS_PROPOSALS_MAX_LIMIT: u64 = 100;

const NNS_PROPOSALS_HELP_AFTER: &str = "\
Examples:
  icq nns proposals
  icq nns proposals --limit 50
  icq nns proposals --before 132000
  icq nns proposals --status open
  icq nns proposals --reward-status settled
  icq nns proposals --topic governance
  icq nns proposals --sort proposed
  icq nns proposals --sort title --asc
  icq nns proposals --format json
  icq nns proposals --source-endpoint https://icp-api.io";

const NNS_PROPOSAL_HELP_AFTER: &str = "\
Examples:
  icq nns proposal 132411
  icq nns proposal 132411 --ballots
  icq nns proposal 132411 --verbose
  icq nns proposal 132411 --format json
  icq nns proposal 132411 --source-endpoint https://icp-api.io";

pub(in crate::nns::proposals) fn nns_proposals_command() -> ClapCommand {
    ClapCommand::new("proposals")
        .bin_name("icq nns proposals")
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
                .default_value(NNS_PROPOSALS_DEFAULT_LIMIT)
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..=NNS_PROPOSALS_MAX_LIMIT))
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
            value_arg(NNS_PROPOSALS_REWARD_STATUS_ARG)
                .long(NNS_PROPOSALS_REWARD_STATUS_ARG)
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
            value_arg("sort")
                .long("sort")
                .value_name(NNS_PROPOSALS_SORT_VALUE_NAME)
                .default_value(NNS_PROPOSAL_SORT_API_LABEL)
                .value_parser(clap::value_parser!(NnsProposalsSortArg))
                .help("Sort proposals locally; status/text sorts default ascending, numeric and timestamp sorts default descending"),
        )
        .arg(
            flag_arg(NNS_PROPOSAL_SORT_ASC_LABEL)
                .long(NNS_PROPOSAL_SORT_ASC_LABEL)
                .conflicts_with(NNS_PROPOSAL_SORT_DESC_LABEL)
                .help("Sort ascending for local sort modes; this is the default for status/topic/proposer/title/action"),
        )
        .arg(
            flag_arg(NNS_PROPOSAL_SORT_DESC_LABEL)
                .long(NNS_PROPOSAL_SORT_DESC_LABEL)
                .conflicts_with(NNS_PROPOSAL_SORT_ASC_LABEL)
                .help("Sort descending for local sort modes; this is the default for id/tally/ballots/reject-cost/reward-round/timestamps"),
        )
        .arg(
            flag_arg(NNS_PROPOSAL_VERBOSE_FLAG)
                .long(NNS_PROPOSAL_VERBOSE_FLAG)
                .help("Show per-proposal detail lines in text output"),
        )
        .arg(leaf::network_arg())
        .after_help(NNS_PROPOSALS_HELP_AFTER)
}

pub(in crate::nns::proposals) fn nns_proposal_command() -> ClapCommand {
    ClapCommand::new("proposal")
        .bin_name("icq nns proposal")
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
        .after_help(NNS_PROPOSAL_HELP_AFTER)
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposals_usage() -> String {
    render_help(nns_proposals_command())
}

pub(in crate::nns::proposals) fn nns_proposals_usage_for_error() -> String {
    render_help(nns_proposals_command())
}

#[cfg(test)]
pub(in crate::nns) fn nns_proposal_usage() -> String {
    render_help(nns_proposal_command())
}

pub(in crate::nns::proposals) fn nns_proposal_usage_for_error() -> String {
    render_help(nns_proposal_command())
}
