//! Module: sns::commands::spec::commands::proposals
//!
//! Responsibility: build clap specs for SNS proposal and proposal-cache commands.
//! Does not own: option parsing, proposal cache behavior, or reports.
//! Boundary: defines proposal command shape, limits, and help examples.

use crate::{
    cli::{
        clap::{flag_arg, value_arg},
        common::{
            COLLECTION_MODE_CACHE_ONLY, COLLECTION_MODE_CACHE_PREFERRED_LIVE_FALLBACK,
            COLLECTION_MODE_CACHE_REFRESH_MISSING, COLLECTION_MODE_FORCE_REFRESH, collection_help,
            json_arg, source_endpoint_arg,
        },
    },
    sns::commands::spec::{
        commands::args::sns_lookup_input_arg,
        values::{
            SNS_PROPOSALS_SORT_VALUE_NAME, SnsProposalEligibilityArg, SnsProposalStatusArg,
            SnsProposalTopicArg, SnsProposalsSortArg,
        },
    },
};
use clap::builder::NonEmptyStringValueParser;
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::sns::{DEFAULT_SNS_SOURCE_ENDPOINT, SNS_REFRESH_MAX_PAGE_SIZE};

const SNS_PROPOSALS_DEFAULT_LIMIT: &str = "25";
const SNS_PROPOSALS_MAX_LIMIT: u64 = 100;
const SNS_PROPOSALS_REFRESH_DEFAULT_PAGE_SIZE: &str = "100";

const SNS_PROPOSALS_HELP_AFTER: &str = "\
Examples:
  icq sns proposal list 1
  icq sns proposal list 1 --status open
  icq sns proposal list 1 --status decided
  icq sns proposal list 1 --topic governance
  icq sns proposal list 1 --eligible yes
  icq sns proposal list 1 --proposer 00010203
  icq sns proposal list 1 --query treasury
  icq sns proposal list 1 --sort status
  icq sns proposal list 1 --sort topic
  icq sns proposal list 1 --sort proposer
  icq sns proposal list 1 --sort title
  icq sns proposal list 1 --sort action
  icq sns proposal list 1 --sort action-id
  icq sns proposal list 1 --sort total-votes
  icq sns proposal list 1 --sort tally-time
  icq sns proposal list 1 --sort ballots
  icq sns proposal list 1 --sort eligible
  icq sns proposal list 1 --sort reject-cost
  icq sns proposal list 1 --sort reward-round
  icq sns proposal list 1 --sort reward-end
  icq sns proposal list 1 --sort created
  icq sns proposal list 1 --sort decided
  icq sns proposal list 1 --sort executed
  icq sns proposal list 1 --sort failed
  icq sns proposal list 1 --sort created --asc
  icq sns proposal refresh 1
  icq sns proposal cache status 1
  icq sns proposal list 1 --before 100 --limit 50
  icq sns proposal list 23ten-uaaaa-aaaaq-aabia-cai --verbose
  icq --network ic sns proposal list 1 --json";

const SNS_PROPOSAL_HELP_AFTER: &str = "\
Examples:
  icq sns proposal info 1 387
  icq sns proposal info 23ten-uaaaa-aaaaq-aabia-cai 387
  icq sns proposal info 1 387 --ballots
  icq sns proposal info 1 387 --verbose
  icq --network ic sns proposal info 1 387 --json";

const SNS_PROPOSALS_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq sns proposal refresh 1
  icq sns proposal refresh 23ten-uaaaa-aaaaq-aabia-cai
  icq sns proposal refresh 1 --page-size 100
  icq --network ic sns proposal refresh 1 --json";

const SNS_PROPOSALS_CACHE_HELP_AFTER: &str = "\
Examples:
  icq sns proposal cache list
  icq sns proposal cache status 1
  icq sns proposal cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns proposal cache status 1 --json";

const SNS_PROPOSALS_CACHE_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns proposal cache list
  icq sns proposal cache list --json";

const SNS_PROPOSALS_CACHE_STATUS_HELP_AFTER: &str = "\
Examples:
  icq sns proposal cache status 1
  icq sns proposal cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns proposal cache status 1 --json";

pub(in crate::sns::commands) fn sns_proposal_command() -> ClapCommand {
    ClapCommand::new("proposal")
        .bin_name("icq sns proposal")
        .about("List, inspect, and refresh SNS governance proposals")
        .subcommand(sns_proposal_list_command())
        .subcommand(sns_proposal_info_command())
        .subcommand(sns_proposal_refresh_command())
        .subcommand(sns_proposal_cache_command())
}

pub(in crate::sns::commands) fn sns_proposal_info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq sns proposal info")
        .about("Show one SNS governance proposal by SNS list id or root principal")
        .arg(sns_lookup_input_arg())
        .arg(
            value_arg("proposal-id")
                .value_name("proposal-id")
                .required(true)
                .value_parser(RangedU64ValueParser::<u64>::new().range(1..))
                .help("SNS governance proposal id"),
        )
        .arg(json_arg())
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
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_PREFERRED_LIVE_FALLBACK,
            SNS_PROPOSAL_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_proposal_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns proposal list")
        .about("List SNS governance proposals by list id or root principal")
        .arg(sns_lookup_input_arg())
        .arg(json_arg())
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
                .value_name("any|open|decided|rejected|adopted|executed|failed")
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
            value_arg("eligible")
                .long("eligible")
                .value_name("any|yes|no")
                .default_value("any")
                .value_parser(clap::value_parser!(SnsProposalEligibilityArg))
                .help("Reward eligibility filter"),
        )
        .arg(
            value_arg("proposer")
                .long("proposer")
                .value_name("neuron-id-prefix")
                .value_parser(NonEmptyStringValueParser::new())
                .help("Filter proposals by proposer neuron id prefix"),
        )
        .arg(
            value_arg("query")
                .long("query")
                .value_name("text")
                .value_parser(NonEmptyStringValueParser::new())
                .help("Case-insensitive title, action, summary, URL, or payload text filter"),
        )
        .arg(
            value_arg("sort")
                .long("sort")
                .value_name(SNS_PROPOSALS_SORT_VALUE_NAME)
                .default_value("api")
                .value_parser(clap::value_parser!(SnsProposalsSortArg))
                .help("Sort proposals locally; status/topic/text sorts default ascending, numeric and timestamp sorts default descending"),
        )
        .arg(
            flag_arg("asc")
                .long("asc")
                .conflicts_with("desc")
                .help("Sort ascending for local sort modes; this is the default for status/topic/proposer/title/action"),
        )
        .arg(
            flag_arg("desc")
                .long("desc")
                .help("Sort descending for local sort modes; this is the default for id/action-id/tally/tally-time/eligible/ballots/reject-cost/reward-round/timestamps"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full proposal titles and per-proposal detail lines in text output"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_REFRESH_MISSING,
            SNS_PROPOSALS_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_proposal_refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq sns proposal refresh")
        .about("Force-refresh and cache a complete SNS governance proposal snapshot")
        .arg(sns_lookup_input_arg())
        .arg(json_arg())
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
                        .range(1..=u64::from(SNS_REFRESH_MAX_PAGE_SIZE)),
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
        .after_help(collection_help(
            COLLECTION_MODE_FORCE_REFRESH,
            SNS_PROPOSALS_REFRESH_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_proposal_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq sns proposal cache")
        .about("Inspect local complete SNS governance proposal snapshots")
        .subcommand(sns_proposal_cache_list_command())
        .subcommand(sns_proposal_cache_status_command())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_PROPOSALS_CACHE_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_proposal_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns proposal cache list")
        .about("List local complete SNS proposal snapshots")
        .arg(json_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_PROPOSALS_CACHE_LIST_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_proposal_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .bin_name("icq sns proposal cache status")
        .about("Show local SNS proposal snapshot and refresh-attempt status")
        .arg(sns_lookup_input_arg())
        .arg(json_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_PROPOSALS_CACHE_STATUS_HELP_AFTER,
        ))
}
