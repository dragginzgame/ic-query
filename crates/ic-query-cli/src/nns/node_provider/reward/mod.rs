//! Module: nns::node_provider::reward
//!
//! Responsibility: parse and dispatch official Dashboard node-provider reward commands.
//! Does not own: Dashboard transport, source validation, report construction, or rendering.
//! Boundary: exposes one page, one exact reward record, or one bounded aggregate history window.

use crate::cli::common::OutputFormat;
use crate::{
    cli::{
        clap::{required_typed, typed_option, value_arg},
        common::{
            COLLECTION_MODE_LIVE, SourceOutputOptions, collection_help, current_unix_secs,
            index_ceiling_arg, json_arg, page_limit_arg, page_offset_arg, resolve_time_bounds,
            source_endpoint_arg, write_text_or_json,
        },
    },
    nns::NnsCommandError,
};
use clap::{ArgMatches, Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::ic::DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT;
use ic_query::ic::DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS;
use ic_query::ic::DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_WINDOW_SECS;
use ic_query::ic::IcNodeProviderRewardHistoryQuery;
use ic_query::ic::IcNodeProviderRewardHistoryRequest;
use ic_query::ic::IcNodeProviderRewardInfoRequest;
use ic_query::ic::IcNodeProviderRewardListQuery;
use ic_query::ic::IcNodeProviderRewardListRequest;
use ic_query::ic::MAX_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS;
use ic_query::ic::MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT;
use ic_query::ic::MIN_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS;
use ic_query::ic::build_ic_node_provider_reward_history_report;
use ic_query::ic::build_ic_node_provider_reward_info_report;
use ic_query::ic::build_ic_node_provider_reward_list_report;
use ic_query_text::ic::ic_node_provider_reward_history_report_text;
use ic_query_text::ic::ic_node_provider_reward_info_report_text;
use ic_query_text::ic::ic_node_provider_reward_list_report_text;

const DEFAULT_LIMIT_ARG: &str = "50";

const INFO_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider reward info 7562
  icq nns node-provider reward info 7562 --json

This command makes exactly one official Dashboard detail request by reward id.
It preserves amounts as raw ICP e8s, timestamps as Unix seconds, and mode-specific
details as JSON. The off-chain response is not certified and is not cached.";

const LIST_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider reward list --limit 25
  icq nns node-provider reward list --offset 25 --max-reward-index 6470 --json

This command makes exactly one official Dashboard list request for at most 100
reward records. It never follows an offset automatically. Reuse the returned
resolved_max_reward_index as --max-reward-index to pin the selected ceiling.
Adjacent upstream offset pages can overlap even when that ceiling is pinned, so
next_offset_hint is arithmetic guidance, not a completeness guarantee. The
off-chain response is not certified and is not cached.";

const HISTORY_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider reward history
  icq nns node-provider reward history --start 1752537600 --end 1784073600 --step 86400 --json

This command makes exactly one official Dashboard aggregate-history request.
The default is the preceding 365 days at a one-day step, and every request is
capped at 1000 requested observations. Amounts remain raw ICP e8s and timestamps
remain Unix seconds. The off-chain response is not certified and is not cached.";

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("reward")
        .bin_name("icq nns node-provider reward")
        .about("Inspect official Dashboard node-provider reward records")
        .subcommand(history_command())
        .subcommand(info_command())
        .subcommand(list_command())
}

pub(super) async fn run(matches: &ArgMatches, _network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("history", matches)) => run_history(matches).await,
        Some(("info", matches)) => run_info(matches).await,
        Some(("list", matches)) => run_list(matches).await,
        _ => unreachable!("clap requires a known nns node-provider reward subcommand"),
    }
}

async fn run_info(matches: &ArgMatches) -> Result<(), NnsCommandError> {
    let (format, request) = info_request(matches, current_unix_secs()?);
    let report = build_ic_node_provider_reward_info_report(&request).await?;
    write_text_or_json(format, &report, ic_node_provider_reward_info_report_text)
}

async fn run_list(matches: &ArgMatches) -> Result<(), NnsCommandError> {
    let (format, request) = list_request(matches, current_unix_secs()?);
    let report = build_ic_node_provider_reward_list_report(&request).await?;
    write_text_or_json(format, &report, ic_node_provider_reward_list_report_text)
}

async fn run_history(matches: &ArgMatches) -> Result<(), NnsCommandError> {
    let (format, request) = history_request(matches, current_unix_secs()?);
    let report = build_ic_node_provider_reward_history_report(&request).await?;
    write_text_or_json(format, &report, ic_node_provider_reward_history_report_text)
}

fn info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq nns node-provider reward info")
        .about("Show one exact Dashboard node-provider reward record")
        .arg(
            value_arg("reward-id")
                .required(true)
                .value_name("reward-id")
                .value_parser(clap::value_parser!(u64))
                .help("Exact numeric Dashboard reward record id"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API v3 base endpoint"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, INFO_HELP_AFTER))
}

fn list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq nns node-provider reward list")
        .about("Show one bounded Dashboard node-provider reward page")
        .arg(
            page_limit_arg(DEFAULT_LIMIT_ARG, MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT)
                .help("Maximum reward records; one through 100"),
        )
        .arg(page_offset_arg().help("Zero-based reward-record offset"))
        .arg(
            index_ceiling_arg("max-reward-index")
                .help("Optional reward-index ceiling returned by an earlier page"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API v3 base endpoint"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER))
}

fn history_command() -> ClapCommand {
    ClapCommand::new("history")
        .bin_name("icq nns node-provider reward history")
        .about("Show bounded aggregate Dashboard node-provider reward history")
        .arg(
            value_arg("start")
                .long("start")
                .value_name("unix-seconds")
                .value_parser(clap::value_parser!(u64))
                .help("Inclusive start; defaults to 365 days before end"),
        )
        .arg(
            value_arg("end")
                .long("end")
                .value_name("unix-seconds")
                .value_parser(clap::value_parser!(u64))
                .help("Inclusive end; defaults to the current time"),
        )
        .arg(
            value_arg("step")
                .long("step")
                .value_name("seconds")
                .default_value(DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS.to_string())
                .hide_default_value(true)
                .value_parser(RangedU64ValueParser::<u32>::new().range(
                    u64::from(MIN_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS)
                        ..=u64::from(MAX_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS),
                ))
                .help("Observation step in seconds; defaults to 86400"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API v3 base endpoint"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, HISTORY_HELP_AFTER))
}

fn info_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcNodeProviderRewardInfoRequest) {
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let request =
        IcNodeProviderRewardInfoRequest::new(dashboard, required_typed(matches, "reward-id"));
    (format, request)
}

fn list_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcNodeProviderRewardListRequest) {
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let request = IcNodeProviderRewardListRequest::new(
        dashboard,
        IcNodeProviderRewardListQuery::new(
            required_typed(matches, "limit"),
            required_typed(matches, "offset"),
            typed_option(matches, "max-reward-index"),
        ),
    );
    (format, request)
}

fn history_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcNodeProviderRewardHistoryRequest) {
    let (start, end) = resolve_time_bounds(
        typed_option(matches, "start"),
        typed_option(matches, "end"),
        now_unix_secs,
        DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_WINDOW_SECS,
        0,
    );
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let request = IcNodeProviderRewardHistoryRequest::new(
        dashboard,
        IcNodeProviderRewardHistoryQuery::new(start, end, required_typed(matches, "step")),
    );
    (format, request)
}

#[cfg(test)]
mod tests;
