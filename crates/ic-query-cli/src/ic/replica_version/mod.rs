//! Module: ic::replica_version
//!
//! Responsibility: parse and dispatch official Dashboard replica-version commands.
//! Does not own: Dashboard transport, source validation, report construction, or rendering.
//! Boundary: exposes one bounded page and one exact release lookup to the IC CLI facade.

use super::IcCommandError;
use crate::cli::clap::{parsed_options, with_options};
use crate::cli::common::OutputFormat;
use crate::cli::common::{
    COLLECTION_MODE_LIVE, collection_help, current_unix_secs, write_text_or_json,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT;
use ic_query::ic::IcReplicaVersionInfoRequest;
use ic_query::ic::IcReplicaVersionListQuery;
use ic_query::ic::IcReplicaVersionListRequest;
use ic_query::ic::MAX_IC_REPLICA_VERSION_PAGE_LIMIT;
use ic_query::ic::build_ic_replica_version_info_report;
use ic_query::ic::build_ic_replica_version_list_report;
use ic_query_text::ic::ic_replica_version_info_report_text;
use ic_query_text::ic::ic_replica_version_list_report_text;

const DEFAULT_LIMIT_ARG: &str = "50";

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

pub(super) async fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches).await,
        Some(("list", matches)) => run_list(matches).await,
        _ => unreachable!("clap requires a known ic replica-version subcommand"),
    }
}

async fn run_info(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let (format, request) = info_request(matches, current_unix_secs()?);
    let report = build_ic_replica_version_info_report(&request).await?;
    write_text_or_json(format, &report, ic_replica_version_info_report_text)
}

async fn run_list(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let (format, request) = list_request(matches, current_unix_secs()?);
    let report = build_ic_replica_version_list_report(&request).await?;
    write_text_or_json(format, &report, ic_replica_version_list_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("replica-version")
        .about("Inspect official Dashboard replica release records")
        .subcommand(info_command())
        .subcommand(list_command())
}

///
/// ReplicaInfoOptions
///
/// Typed arguments for the info_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct ReplicaInfoOptions {
    #[arg(
        id = "replica-version-id",
        value_name = "replica-version-id",
        help = "Exact 40-character lowercase hexadecimal version id"
    )]
    pub(super) replica_version_id: String,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

fn info_command() -> ClapCommand {
    with_options::<ReplicaInfoOptions>(
        ClapCommand::new("info")
            .about("Show one exact Dashboard replica release record")
            .after_help(collection_help(COLLECTION_MODE_LIVE, INFO_HELP_AFTER)),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
            .help("Official IC Dashboard API v3 base endpoint")
    })
}

///
/// ReplicaListOptions
///
/// Typed arguments for the list_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct ReplicaListOptions {
    #[arg(long, value_name = "rows", default_value = DEFAULT_LIMIT_ARG, value_parser = clap::builder::RangedU64ValueParser::<u16>::new().range(1..=u64::from(MAX_IC_REPLICA_VERSION_PAGE_LIMIT)), help = "Maximum release rows; one through 100")]
    pub(super) limit: u16,
    #[arg(
        long,
        value_name = "rows",
        default_value = "0",
        help = "Zero-based release-row offset"
    )]
    pub(super) offset: u64,
    #[arg(
        id = "max-proposal-index",
        long,
        value_name = "index",
        help = "Optional proposal-index ceiling returned by an earlier page"
    )]
    pub(super) max_proposal_index: Option<u64>,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

fn list_command() -> ClapCommand {
    with_options::<ReplicaListOptions>(
        ClapCommand::new("list")
            .about("Show one bounded Dashboard replica release page")
            .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER)),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
            .help("Official IC Dashboard API v3 base endpoint")
    })
}

fn info_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcReplicaVersionInfoRequest) {
    let options: ReplicaInfoOptions = parsed_options(matches);
    let (format, dashboard) = options.source.into_dashboard_request(now_unix_secs);
    let request = IcReplicaVersionInfoRequest::new(dashboard, options.replica_version_id);
    (format, request)
}

fn list_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcReplicaVersionListRequest) {
    let options: ReplicaListOptions = parsed_options(matches);
    let (format, dashboard) = options.source.into_dashboard_request(now_unix_secs);
    let request = IcReplicaVersionListRequest::new(
        dashboard,
        IcReplicaVersionListQuery::new(options.limit, options.offset, options.max_proposal_index),
    );
    (format, request)
}

#[cfg(test)]
mod tests;
