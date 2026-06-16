use super::super::values::{SnsProposalStatusArg, SnsProposalTopicArg};
use super::args::sns_lookup_input_arg;
use crate::{
    cli::{
        clap::{flag_arg, value_arg},
        common::{format_arg, source_endpoint_arg},
        globals::internal_network_arg,
    },
    sns::report::DEFAULT_SNS_SOURCE_ENDPOINT,
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};

const SNS_PROPOSALS_DEFAULT_LIMIT: &str = "25";
const SNS_PROPOSALS_MAX_LIMIT: u64 = 100;

const SNS_PROPOSALS_HELP_AFTER: &str = "\
Examples:
  icq sns proposals 1
  icq sns proposals 1 --status open
  icq sns proposals 1 --topic governance
  icq sns proposals 1 --before 100 --limit 50
  icq sns proposals 23ten-uaaaa-aaaaq-aabia-cai --verbose
  icq --network ic sns proposals 1 --format json";

const SNS_PROPOSAL_HELP_AFTER: &str = "\
Examples:
  icq sns proposal 1 387
  icq sns proposal 23ten-uaaaa-aaaaq-aabia-cai 387
  icq sns proposal 1 387 --ballots
  icq sns proposal 1 387 --verbose
  icq --network ic sns proposal 1 387 --format json";

pub(in crate::sns::commands) fn sns_proposal_command() -> ClapCommand {
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
        .arg(
            flag_arg("ballots")
                .long("ballots")
                .help("Show proposal ballot rows in text output"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PROPOSAL_HELP_AFTER)
}

pub(in crate::sns::commands) fn sns_proposals_command() -> ClapCommand {
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
            value_arg("topic")
                .long("topic")
                .value_name("topic")
                .default_value("any")
                .value_parser(clap::value_parser!(SnsProposalTopicArg))
                .help("SNS governance topic filter"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full proposal titles and per-proposal detail lines in text output"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PROPOSALS_HELP_AFTER)
}
