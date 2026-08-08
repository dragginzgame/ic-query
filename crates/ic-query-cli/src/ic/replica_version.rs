//! Module: ic::replica_version
//!
//! Responsibility: parse and dispatch official Dashboard replica-version commands.
//! Does not own: Dashboard transport, source validation, report construction, or rendering.
//! Boundary: exposes one bounded page and one exact release lookup to the IC CLI facade.

use super::IcCommandError;
#[cfg(test)]
use super::parse_test_options;
use crate::cli::{
    clap::{required_string, required_typed, typed_option, value_arg},
    common::{
        COLLECTION_MODE_LIVE, OutputFormat, collection_help, current_unix_secs, json_arg,
        output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand, builder::RangedU64ValueParser};
#[cfg(test)]
use ic_query::ic::DEFAULT_IC_REPLICA_VERSION_PAGE_LIMIT;
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcReplicaVersionInfoRequest, IcReplicaVersionListQuery,
    IcReplicaVersionListRequest, MAX_IC_REPLICA_VERSION_PAGE_LIMIT,
    build_ic_replica_version_info_report, build_ic_replica_version_list_report,
    ic_replica_version_info_report_text, ic_replica_version_list_report_text,
};

const DEFAULT_LIMIT_ARG: &str = "50";
const DEFAULT_OFFSET_ARG: &str = "0";

const INFO_HELP_AFTER: &str = "\
Examples:
  icq ic replica-version info e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3
  icq ic replica-version info e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3 --json

This command makes exactly one official Dashboard detail request. It reports
release-election and Dashboard-recorded Subnet rollout evidence; it does not
prove which replica version a Subnet is currently running. No cache is used.";

const LIST_HELP_AFTER: &str = "\
Examples:
  icq ic replica-version list --limit 25
  icq ic replica-version list --offset 25 --max-proposal-index 438 --json

This command makes exactly one official Dashboard list request for at most 100
release rows. It never follows an offset automatically. Reuse the returned
resolved_max_proposal_index as --max-proposal-index when requesting a later
page so newly indexed proposals do not move the selected ceiling. The response
is off-chain, non-certified, not runtime-version evidence, and is not cached.";

pub(super) fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches),
        Some(("list", matches)) => run_list(matches),
        _ => unreachable!("clap requires a known ic replica-version subcommand"),
    }
}

fn run_info(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = ReplicaVersionInfoOptions::from_matches(matches);
    let request = IcReplicaVersionInfoRequest::new(
        options.source_endpoint,
        current_unix_secs()?,
        options.replica_version_id,
    );
    let report = build_ic_replica_version_info_report(&request)?;
    write_text_or_json(options.format, &report, ic_replica_version_info_report_text)
}

fn run_list(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = ReplicaVersionListOptions::from_matches(matches);
    let request = IcReplicaVersionListRequest::new(
        options.source_endpoint,
        current_unix_secs()?,
        IcReplicaVersionListQuery::new(options.limit, options.offset, options.max_proposal_index),
    );
    let report = build_ic_replica_version_list_report(&request)?;
    write_text_or_json(options.format, &report, ic_replica_version_list_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("replica-version")
        .bin_name("icq ic replica-version")
        .about("Inspect official Dashboard replica release records")
        .subcommand(info_command())
        .subcommand(list_command())
}

fn info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq ic replica-version info")
        .about("Show one exact Dashboard replica release record")
        .arg(
            value_arg("replica-version-id")
                .required(true)
                .value_name("replica-version-id")
                .help("Exact 40-character lowercase hexadecimal version id"),
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
        .bin_name("icq ic replica-version list")
        .about("Show one bounded Dashboard replica release page")
        .arg(
            value_arg("limit")
                .long("limit")
                .value_name("rows")
                .default_value(DEFAULT_LIMIT_ARG)
                .value_parser(
                    RangedU64ValueParser::<u16>::new()
                        .range(1..=u64::from(MAX_IC_REPLICA_VERSION_PAGE_LIMIT)),
                )
                .help("Maximum release rows; one through 100"),
        )
        .arg(
            value_arg("offset")
                .long("offset")
                .value_name("rows")
                .default_value(DEFAULT_OFFSET_ARG)
                .value_parser(clap::value_parser!(u64))
                .help("Zero-based release-row offset"),
        )
        .arg(
            value_arg("max-proposal-index")
                .long("max-proposal-index")
                .value_name("index")
                .value_parser(clap::value_parser!(u64))
                .help("Optional proposal-index ceiling returned by an earlier page"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API v3 base endpoint"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplicaVersionInfoOptions {
    replica_version_id: String,
    format: OutputFormat,
    source_endpoint: String,
}

impl ReplicaVersionInfoOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            replica_version_id: required_string(matches, "replica-version-id"),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplicaVersionListOptions {
    limit: u16,
    offset: u64,
    max_proposal_index: Option<u64>,
    format: OutputFormat,
    source_endpoint: String,
}

impl ReplicaVersionListOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            limit: required_typed(matches, "limit"),
            offset: required_typed(matches, "offset"),
            max_proposal_index: typed_option(matches, "max-proposal-index"),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::render_help;

    const VERSION_ID: &str = "e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3";

    #[test]
    fn usage_discloses_bounds_authority_and_runtime_limit() {
        let family = render_help(command());
        let info = render_help(info_command());
        let list = render_help(list_command());

        assert!(family.contains("Usage: icq ic replica-version [COMMAND]"));
        assert!(family.contains("info"));
        assert!(family.contains("list"));
        assert!(info.contains("exactly one official Dashboard detail request"));
        assert!(info.contains("prove which replica version"));
        assert!(list.contains("release rows"));
        assert!(list.contains("--max-proposal-index"));
        assert!(list.contains("not runtime-version evidence"));
    }

    #[test]
    fn info_options_preserve_exact_target_format_and_endpoint() {
        let options = parse_test_options(
            info_command(),
            &[
                VERSION_ID,
                "--json",
                "--source-endpoint",
                "https://example.com/api/v3",
            ],
            ReplicaVersionInfoOptions::from_matches,
        )
        .expect("info options");

        assert_eq!(options.replica_version_id, VERSION_ID);
        assert_eq!(options.format, OutputFormat::Json);
        assert_eq!(options.source_endpoint, "https://example.com/api/v3");
    }

    #[test]
    fn list_options_preserve_explicit_page_bounds() {
        let options = parse_test_options(
            list_command(),
            &[
                "--limit",
                "25",
                "--offset",
                "50",
                "--max-proposal-index",
                "438",
                "--json",
            ],
            ReplicaVersionListOptions::from_matches,
        )
        .expect("list options");

        assert_eq!(options.limit, 25);
        assert_eq!(options.offset, 50);
        assert_eq!(options.max_proposal_index, Some(438));
        assert_eq!(options.format, OutputFormat::Json);
    }

    #[test]
    fn list_options_use_bounded_defaults() {
        let options =
            parse_test_options(list_command(), &[], ReplicaVersionListOptions::from_matches)
                .expect("default list options");

        assert_eq!(options.limit, DEFAULT_IC_REPLICA_VERSION_PAGE_LIMIT);
        assert_eq!(options.offset, 0);
        assert_eq!(options.max_proposal_index, None);
        assert_eq!(options.format, OutputFormat::Text);
    }
}
