//! Module: nns::node_provider::reward
//!
//! Responsibility: parse and dispatch official Dashboard node-provider reward commands.
//! Does not own: Dashboard transport, source validation, report construction, or rendering.
//! Boundary: exposes one page, one exact reward record, or one bounded aggregate history window.

use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option, value_arg},
        common::{
            COLLECTION_MODE_LIVE, OutputFormat, collection_help, json_arg, output_format,
            source_endpoint_arg, write_text_or_json,
        },
    },
    nns::{NnsCommandError, now_unix_secs},
};
use clap::{ArgMatches, Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS,
    DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_WINDOW_SECS, IcNodeProviderRewardHistoryQuery,
    IcNodeProviderRewardHistoryRequest, IcNodeProviderRewardInfoRequest,
    IcNodeProviderRewardListQuery, IcNodeProviderRewardListRequest,
    MAX_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS, MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT,
    MIN_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS, build_ic_node_provider_reward_history_report,
    build_ic_node_provider_reward_info_report, build_ic_node_provider_reward_list_report,
    ic_node_provider_reward_history_report_text, ic_node_provider_reward_info_report_text,
    ic_node_provider_reward_list_report_text,
};

const DEFAULT_LIMIT_ARG: &str = "50";
const DEFAULT_OFFSET_ARG: &str = "0";

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

pub(super) fn run(matches: &ArgMatches, _network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("history", matches)) => run_history(matches),
        Some(("info", matches)) => run_info(matches),
        Some(("list", matches)) => run_list(matches),
        _ => unreachable!("clap requires a known nns node-provider reward subcommand"),
    }
}

fn run_info(matches: &ArgMatches) -> Result<(), NnsCommandError> {
    let options = RewardInfoOptions::from_matches(matches);
    let request = IcNodeProviderRewardInfoRequest::new(
        options.source_endpoint,
        now_unix_secs()?,
        options.reward_id,
    );
    let report = build_ic_node_provider_reward_info_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        ic_node_provider_reward_info_report_text,
    )
}

fn run_list(matches: &ArgMatches) -> Result<(), NnsCommandError> {
    let options = RewardListOptions::from_matches(matches);
    let request = IcNodeProviderRewardListRequest::new(
        options.source_endpoint,
        now_unix_secs()?,
        IcNodeProviderRewardListQuery::new(options.limit, options.offset, options.max_reward_index),
    );
    let report = build_ic_node_provider_reward_list_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        ic_node_provider_reward_list_report_text,
    )
}

fn run_history(matches: &ArgMatches) -> Result<(), NnsCommandError> {
    let options = RewardHistoryOptions::from_matches(matches);
    let now = now_unix_secs()?;
    let end = options.end_unix_secs.unwrap_or(now);
    let start = options
        .start_unix_secs
        .unwrap_or_else(|| end.saturating_sub(DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_WINDOW_SECS));
    let request = IcNodeProviderRewardHistoryRequest::new(
        options.source_endpoint,
        now,
        IcNodeProviderRewardHistoryQuery::new(start, end, options.step_secs),
    );
    let report = build_ic_node_provider_reward_history_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        ic_node_provider_reward_history_report_text,
    )
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
            value_arg("limit")
                .long("limit")
                .value_name("rows")
                .default_value(DEFAULT_LIMIT_ARG)
                .value_parser(
                    RangedU64ValueParser::<u16>::new()
                        .range(1..=u64::from(MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT)),
                )
                .help("Maximum reward records; one through 100"),
        )
        .arg(
            value_arg("offset")
                .long("offset")
                .value_name("rows")
                .default_value(DEFAULT_OFFSET_ARG)
                .value_parser(clap::value_parser!(u64))
                .help("Zero-based reward-record offset"),
        )
        .arg(
            value_arg("max-reward-index")
                .long("max-reward-index")
                .value_name("index")
                .value_parser(clap::value_parser!(u64))
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct RewardInfoOptions {
    reward_id: u64,
    format: OutputFormat,
    source_endpoint: String,
}

impl RewardInfoOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            reward_id: required_typed(matches, "reward-id"),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RewardListOptions {
    limit: u16,
    offset: u64,
    max_reward_index: Option<u64>,
    format: OutputFormat,
    source_endpoint: String,
}

impl RewardListOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            limit: required_typed(matches, "limit"),
            offset: required_typed(matches, "offset"),
            max_reward_index: typed_option(matches, "max-reward-index"),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RewardHistoryOptions {
    start_unix_secs: Option<u64>,
    end_unix_secs: Option<u64>,
    step_secs: u32,
    format: OutputFormat,
    source_endpoint: String,
}

impl RewardHistoryOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            start_unix_secs: typed_option(matches, "start"),
            end_unix_secs: typed_option(matches, "end"),
            step_secs: typed_option(matches, "step")
                .unwrap_or(DEFAULT_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::{parse_matches, render_help};

    fn options<T>(command: ClapCommand, args: &[&str], from: impl FnOnce(&ArgMatches) -> T) -> T {
        let matches = parse_matches(command, args.iter().map(std::ffi::OsString::from))
            .expect("valid options");
        from(&matches)
    }

    #[test]
    fn help_discloses_authority_bounds_and_page_overlap() {
        let family = render_help(command());
        let list = render_help(list_command());
        let history = render_help(history_command());

        assert!(family.contains("Usage: icq nns node-provider reward [COMMAND]"));
        assert!(family.contains("history"));
        assert!(family.contains("info"));
        assert!(family.contains("list"));
        assert!(list.contains("Adjacent upstream offset pages can overlap"));
        assert!(list.contains("--max-reward-index"));
        assert!(history.contains("capped at 1000 requested observations"));
    }

    #[test]
    fn options_preserve_explicit_bounds_and_raw_format() {
        let list = options(
            list_command(),
            &[
                "--limit",
                "25",
                "--offset",
                "50",
                "--max-reward-index",
                "6470",
                "--json",
            ],
            RewardListOptions::from_matches,
        );
        assert_eq!(list.limit, 25);
        assert_eq!(list.offset, 50);
        assert_eq!(list.max_reward_index, Some(6_470));
        assert_eq!(list.format, OutputFormat::Json);

        let history = options(
            history_command(),
            &[
                "--start",
                "1752537600",
                "--end",
                "1784073600",
                "--step",
                "86400",
            ],
            RewardHistoryOptions::from_matches,
        );
        assert_eq!(history.start_unix_secs, Some(1_752_537_600));
        assert_eq!(history.end_unix_secs, Some(1_784_073_600));
        assert_eq!(history.step_secs, 86_400);
    }
}
