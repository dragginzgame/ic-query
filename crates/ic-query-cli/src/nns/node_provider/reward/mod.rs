//! Module: nns::node_provider::reward
//!
//! Responsibility: parse and dispatch official Dashboard node-provider reward commands.
//! Does not own: Dashboard transport, source validation, report construction, or rendering.
//! Boundary: exposes one page, one exact reward record, or one bounded aggregate history window.

use crate::cli::clap::{parsed_options, with_options};
use crate::cli::common::OutputFormat;
use crate::{
    cli::common::{
        COLLECTION_MODE_LIVE, collection_help, current_unix_secs, resolve_time_bounds,
        write_text_or_json,
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

///
/// RewardInfoOptions
///
/// Typed arguments for the info_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct RewardInfoOptions {
    #[arg(id = "reward-id", value_name = "reward-id", value_parser = clap::value_parser!(u64), help = "Exact numeric Dashboard reward record id")]
    pub(super) reward_id: u64,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

fn info_command() -> ClapCommand {
    with_options::<RewardInfoOptions>(
        ClapCommand::new("info")
            .about("Show one exact Dashboard node-provider reward record")
            .after_help(collection_help(COLLECTION_MODE_LIVE, INFO_HELP_AFTER)),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
            .help("Official IC Dashboard API v3 base endpoint")
    })
}

///
/// RewardListOptions
///
/// Typed arguments for the list_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct RewardListOptions {
    #[arg(long, value_name = "rows", default_value = DEFAULT_LIMIT_ARG, value_parser = clap::builder::RangedU64ValueParser::<u16>::new().range(1..=u64::from(MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT)), help = "Maximum reward records; one through 100")]
    pub(super) limit: u16,
    #[arg(
        long,
        value_name = "rows",
        default_value = "0",
        help = "Zero-based reward-record offset"
    )]
    pub(super) offset: u64,
    #[arg(
        id = "max-reward-index",
        long,
        value_name = "index",
        help = "Optional reward-index ceiling returned by an earlier page"
    )]
    pub(super) max_reward_index: Option<u64>,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

fn list_command() -> ClapCommand {
    with_options::<RewardListOptions>(
        ClapCommand::new("list")
            .about("Show one bounded Dashboard node-provider reward page")
            .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER)),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
            .help("Official IC Dashboard API v3 base endpoint")
    })
}

///
/// RewardHistoryOptions
///
/// Typed arguments for the history_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct RewardHistoryOptions {
    #[arg(long, value_name = "unix-seconds", value_parser = clap::value_parser!(u64), help = "Inclusive start; defaults to 365 days before end")]
    pub(super) start: Option<u64>,
    #[arg(long, value_name = "unix-seconds", value_parser = clap::value_parser!(u64), help = "Inclusive end; defaults to the current time")]
    pub(super) end: Option<u64>,
    #[arg(long, value_name = "seconds", default_value = DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS.to_string(), hide_default_value = true, value_parser = RangedU64ValueParser::<u32>::new().range( u64::from(MIN_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS) ..=u64::from(MAX_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS), ), help = "Observation step in seconds; defaults to 86400")]
    pub(super) step: u32,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

fn history_command() -> ClapCommand {
    with_options::<RewardHistoryOptions>(
        ClapCommand::new("history")
            .about("Show bounded aggregate Dashboard node-provider reward history")
            .after_help(collection_help(COLLECTION_MODE_LIVE, HISTORY_HELP_AFTER)),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
            .help("Official IC Dashboard API v3 base endpoint")
    })
}

fn info_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcNodeProviderRewardInfoRequest) {
    let options: RewardInfoOptions = parsed_options(matches);
    let (format, dashboard) = options.source.into_dashboard_request(now_unix_secs);
    let request = IcNodeProviderRewardInfoRequest::new(dashboard, options.reward_id);
    (format, request)
}

fn list_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcNodeProviderRewardListRequest) {
    let options: RewardListOptions = parsed_options(matches);
    let (format, dashboard) = options.source.into_dashboard_request(now_unix_secs);
    let request = IcNodeProviderRewardListRequest::new(
        dashboard,
        IcNodeProviderRewardListQuery::new(options.limit, options.offset, options.max_reward_index),
    );
    (format, request)
}

fn history_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcNodeProviderRewardHistoryRequest) {
    let options: RewardHistoryOptions = parsed_options(matches);
    let (start, end) = resolve_time_bounds(
        options.start,
        options.end,
        now_unix_secs,
        DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_WINDOW_SECS,
        0,
    );
    let (format, dashboard) = options.source.into_dashboard_request(now_unix_secs);
    let request = IcNodeProviderRewardHistoryRequest::new(
        dashboard,
        IcNodeProviderRewardHistoryQuery::new(start, end, options.step),
    );
    (format, request)
}

#[cfg(test)]
mod tests;
