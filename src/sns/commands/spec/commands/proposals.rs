//! Module: sns::commands::spec::commands::proposals
//!
//! Responsibility: build clap specs for SNS proposal and proposal-cache commands.
//! Does not own: option parsing, proposal cache behavior, or reports.
//! Boundary: defines proposal command shape, limits, and help examples.

use crate::{
    cli::{
        clap::{flag_arg, passthrough_subcommand, value_arg},
        common::{format_arg, source_endpoint_arg},
        globals::internal_network_arg,
    },
    sns::{
        commands::spec::{
            commands::{args::sns_lookup_input_arg, nested_dispatch_command},
            values::{SnsProposalStatusArg, SnsProposalTopicArg, SnsProposalsSortArg},
        },
        report::DEFAULT_SNS_SOURCE_ENDPOINT,
    },
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};

const SNS_PROPOSALS_DEFAULT_LIMIT: &str = "25";
const SNS_PROPOSALS_MAX_LIMIT: u64 = 100;
const SNS_PROPOSALS_REFRESH_DEFAULT_PAGE_SIZE: &str = "100";
const SNS_PROPOSALS_REFRESH_MAX_PAGE_SIZE: u64 = 100;

const SNS_PROPOSALS_HELP_AFTER: &str = "\
Examples:
  icq sns proposals 1
  icq sns proposals 1 --status open
  icq sns proposals 1 --topic governance
  icq sns proposals 1 --sort created
  icq sns proposals 1 --sort decided
  icq sns proposals 1 --sort executed
  icq sns proposals 1 --sort failed
  icq sns proposals 1 --sort created --asc
  icq sns proposals refresh 1
  icq sns proposals cache status 1
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

const SNS_PROPOSALS_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq sns proposals refresh 1
  icq sns proposals refresh 23ten-uaaaa-aaaaq-aabia-cai
  icq sns proposals refresh 1 --page-size 100
  icq --network ic sns proposals refresh 1 --format json";

const SNS_PROPOSALS_CACHE_HELP_AFTER: &str = "\
Examples:
  icq sns proposals cache list
  icq sns proposals cache status 1
  icq sns proposals cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns proposals cache status 1 --format json";

const SNS_PROPOSALS_CACHE_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns proposals cache list
  icq sns proposals cache list --format json";

const SNS_PROPOSALS_CACHE_STATUS_HELP_AFTER: &str = "\
Examples:
  icq sns proposals cache status 1
  icq sns proposals cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns proposals cache status 1 --format json";

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
            value_arg("sort")
                .long("sort")
                .value_name("api|id|created|decided|executed|failed")
                .default_value("api")
                .value_parser(clap::value_parser!(SnsProposalsSortArg))
                .help("Sort proposals locally; timestamp sorts are newest first"),
        )
        .arg(
            flag_arg("asc")
                .long("asc")
                .conflicts_with("desc")
                .help("Sort ascending for local sort modes"),
        )
        .arg(
            flag_arg("desc")
                .long("desc")
                .conflicts_with("asc")
                .help("Sort descending for local sort modes; this is the default"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full proposal titles and per-proposal detail lines in text output"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PROPOSALS_HELP_AFTER)
}

pub(in crate::sns::commands) fn sns_proposals_dispatch_command() -> ClapCommand {
    nested_dispatch_command(
        "proposals",
        "icq sns proposals",
        "Force-refresh and cache a complete SNS governance proposal snapshot",
        "Inspect local complete SNS governance proposal snapshots",
    )
}

pub(in crate::sns::commands) fn sns_proposals_refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq sns proposals refresh")
        .about("Force-refresh and cache a complete SNS governance proposal snapshot")
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
                .default_value(SNS_PROPOSALS_REFRESH_DEFAULT_PAGE_SIZE)
                .value_parser(
                    RangedU64ValueParser::<u32>::new()
                        .range(1..=SNS_PROPOSALS_REFRESH_MAX_PAGE_SIZE),
                )
                .help("Maximum proposals to request per SNS governance page"),
        )
        .arg(
            value_arg("max-pages")
                .long("max-pages")
                .value_name("count")
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Stop before publishing if this page count is reached before API exhaustion"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PROPOSALS_REFRESH_HELP_AFTER)
}

pub(in crate::sns::commands) fn sns_proposals_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq sns proposals cache")
        .about("Inspect local complete SNS governance proposal snapshots")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List local complete SNS proposal snapshots"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("status").about(
            "Show local SNS proposal snapshot and refresh-attempt status",
        )))
        .after_help(SNS_PROPOSALS_CACHE_HELP_AFTER)
}

pub(in crate::sns::commands) fn sns_proposals_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns proposals cache list")
        .about("List local complete SNS proposal snapshots")
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PROPOSALS_CACHE_LIST_HELP_AFTER)
}

pub(in crate::sns::commands) fn sns_proposals_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .bin_name("icq sns proposals cache status")
        .about("Show local SNS proposal snapshot and refresh-attempt status")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_PROPOSALS_CACHE_STATUS_HELP_AFTER)
}
