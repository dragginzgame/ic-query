//! Module: ic::canister
//!
//! Responsibility: parse and dispatch official Dashboard canister commands.
//! Does not own: Dashboard transport, report construction, or text rendering.
//! Boundary: exposes one bounded canister command family to the IC CLI facade.

use super::IcCommandError;
#[cfg(test)]
use super::parse_test_options;
use crate::cli::{
    clap::{required_string, required_typed, string_option, typed_option, value_arg},
    common::{
        COLLECTION_MODE_LIVE, OutputFormat, collection_help, current_unix_secs, json_arg,
        output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgAction, ArgMatches, Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT, DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
    IcCanisterCountRequest, IcCanisterFilters, IcCanisterPageRequest, IcCanisterRequest,
    MAX_IC_CANISTER_PAGE_LIMIT, build_ic_canister_count_report, build_ic_canister_page_report,
    build_ic_canister_report, ic_canister_count_report_text, ic_canister_page_report_text,
    ic_canister_report_text,
};

const CANISTER_INFO_HELP_AFTER: &str = "\
Examples:
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --json
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --source-endpoint https://ic-api.internetcomputer.org/api/v3

The official Dashboard API is an off-chain analytics authority. Its response
is not presented as certified IC state or an exact point-in-time snapshot.";
const DEFAULT_CANISTER_PAGE_LIMIT_ARG: &str = "50";

const CANISTER_COUNT_HELP_AFTER: &str = "\
Examples:
  icq ic canister count
  icq ic canister count --has-name true --canister-type ledger --json

This command makes exactly one official Dashboard count request. It does not
fetch canister rows, follow cursors, or create a cache.";

const CANISTER_PAGE_HELP_AFTER: &str = "\
Examples:
  icq ic canister page --limit 25
  icq ic canister page --query ledger --limit 25 --json
  icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25

This command makes exactly one official Dashboard page request. Results are
ordered by canister id, the limit is capped at 100, and returned cursors are
followed only when supplied explicitly to a later command. No cache is used.";

pub(super) fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches),
        Some(("count", matches)) => run_count(matches),
        Some(("page", matches)) => run_page(matches),
        _ => unreachable!("clap requires a known ic canister subcommand"),
    }
}

fn run_count(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = CanisterCollectionOptions::from_matches(matches);
    let request = IcCanisterCountRequest::new(options.source_endpoint, current_unix_secs()?)
        .with_filters(options.filters);
    let report = build_ic_canister_count_report(&request)?;
    write_text_or_json(options.format, &report, ic_canister_count_report_text)
}

fn run_page(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = CanisterPageOptions::from_matches(matches);
    let mut request =
        IcCanisterPageRequest::new(options.collection.source_endpoint, current_unix_secs()?)
            .with_filters(options.collection.filters)
            .with_limit(options.limit);
    request.after = options.after;
    request.before = options.before;
    let report = build_ic_canister_page_report(&request)?;
    write_text_or_json(
        options.collection.format,
        &report,
        ic_canister_page_report_text,
    )
}

fn run_info(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = CanisterInfoOptions::from_matches(matches);
    let request = IcCanisterRequest::new(
        options.source_endpoint,
        current_unix_secs()?,
        options.canister_id,
    );
    let report = build_ic_canister_report(&request)?;
    write_text_or_json(options.format, &report, ic_canister_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("canister")
        .bin_name("icq ic canister")
        .about("Inspect deployed canister metadata")
        .subcommand(info_command())
        .subcommand(count_command())
        .subcommand(page_command())
}

fn count_command() -> ClapCommand {
    collection_args(
        ClapCommand::new("count")
            .bin_name("icq ic canister count")
            .about("Count canisters through one official Dashboard API request"),
    )
    .after_help(collection_help(
        COLLECTION_MODE_LIVE,
        CANISTER_COUNT_HELP_AFTER,
    ))
}

fn page_command() -> ClapCommand {
    collection_args(
        ClapCommand::new("page")
            .bin_name("icq ic canister page")
            .about("Show one bounded official Dashboard canister page"),
    )
    .arg(
        value_arg("limit")
            .long("limit")
            .value_name("rows")
            .default_value(DEFAULT_CANISTER_PAGE_LIMIT_ARG)
            .value_parser(
                RangedU64ValueParser::<u16>::new().range(1..=u64::from(MAX_IC_CANISTER_PAGE_LIMIT)),
            )
            .help("Maximum rows; one through 100"),
    )
    .arg(
        value_arg("after")
            .long("after")
            .value_name("canister-id")
            .conflicts_with("before")
            .help("Exclusive forward cursor returned by a prior page"),
    )
    .arg(
        value_arg("before")
            .long("before")
            .value_name("canister-id")
            .help("Exclusive backward cursor returned by a prior page"),
    )
    .after_help(collection_help(
        COLLECTION_MODE_LIVE,
        CANISTER_PAGE_HELP_AFTER,
    ))
}

fn collection_args(command: ClapCommand) -> ClapCommand {
    filter_args(command).arg(json_arg()).arg(
        source_endpoint_arg(DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT)
            .help("Official IC Dashboard API v4 base endpoint"),
    )
}

fn filter_args(command: ClapCommand) -> ClapCommand {
    command
        .arg(
            value_arg("has-name")
                .long("has-name")
                .value_name("true|false")
                .value_parser(clap::value_parser!(bool))
                .help("Filter by whether a Dashboard name is recorded"),
        )
        .arg(
            value_arg("subnet-id")
                .long("subnet-id")
                .value_name("principal")
                .help("Filter by Subnet principal"),
        )
        .arg(
            value_arg("controller-id")
                .long("controller-id")
                .value_name("principal")
                .help("Filter by controller principal"),
        )
        .arg(
            value_arg("language")
                .long("language")
                .value_name("label")
                .action(ArgAction::Append)
                .help("Filter by a raw Dashboard language label; repeatable"),
        )
        .arg(
            value_arg("canister-type")
                .long("canister-type")
                .value_name("classification")
                .action(ArgAction::Append)
                .help("Filter by a raw Dashboard canister classification; repeatable"),
        )
        .arg(
            value_arg("query")
                .long("query")
                .value_name("text")
                .help("Dashboard text search; two through 100 characters"),
        )
}

fn info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq ic canister info")
        .about("Show one canister from the official Dashboard API")
        .arg(
            value_arg("canister-id")
                .required(true)
                .value_name("canister-id")
                .help("Canister principal"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API base endpoint"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            CANISTER_INFO_HELP_AFTER,
        ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanisterInfoOptions {
    canister_id: String,
    format: OutputFormat,
    source_endpoint: String,
}

impl CanisterInfoOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            canister_id: required_string(matches, "canister-id"),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanisterCollectionOptions {
    filters: IcCanisterFilters,
    format: OutputFormat,
    source_endpoint: String,
}

impl CanisterCollectionOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            filters: canister_filters(matches),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanisterPageOptions {
    collection: CanisterCollectionOptions,
    limit: u16,
    after: Option<String>,
    before: Option<String>,
}

impl CanisterPageOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            collection: CanisterCollectionOptions::from_matches(matches),
            limit: required_typed(matches, "limit"),
            after: string_option(matches, "after"),
            before: string_option(matches, "before"),
        }
    }
}

fn canister_filters(matches: &ArgMatches) -> IcCanisterFilters {
    IcCanisterFilters {
        has_name: typed_option(matches, "has-name"),
        subnet_id: string_option(matches, "subnet-id"),
        controller_id: string_option(matches, "controller-id"),
        languages: repeated_strings(matches, "language"),
        canister_types: repeated_strings(matches, "canister-type"),
        query: string_option(matches, "query"),
    }
}

fn repeated_strings(matches: &ArgMatches, id: &str) -> Vec<String> {
    matches
        .get_many::<String>(id)
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::render_help;
    use ic_query::ic::IcHostError;
    use std::ffi::OsString;

    const CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

    #[test]
    fn usage_discloses_live_dashboard_authority_and_command_shape() {
        let canister = render_help(command());
        let info = render_help(info_command());
        let count = render_help(count_command());
        let page = render_help(page_command());

        assert!(canister.contains("Usage: icq ic canister [COMMAND]"));
        assert!(canister.contains("info"));
        assert!(canister.contains("count"));
        assert!(canister.contains("page"));
        assert!(info.contains("Usage: icq ic canister info [OPTIONS] <canister-id>"));
        assert!(info.contains("Live query; does not read or write a report cache."));
        assert!(info.contains("off-chain analytics authority"));
        assert!(info.contains("--source-endpoint"));
        assert!(count.contains("Usage: icq ic canister count [OPTIONS]"));
        assert!(count.contains("exactly one official Dashboard count request"));
        assert!(page.contains("Usage: icq ic canister page [OPTIONS]"));
        assert!(page.contains("limit is capped at 100"));
        assert!(page.contains("No cache is used"));
    }

    #[test]
    fn canister_info_options_preserve_principal_format_and_endpoint() {
        let options = parse_test_options(
            info_command(),
            &[
                CANISTER_ID,
                "--json",
                "--source-endpoint",
                "https://example.com/api/v3",
            ],
            CanisterInfoOptions::from_matches,
        )
        .expect("canister options");

        assert_eq!(options.canister_id, CANISTER_ID);
        assert_eq!(options.format, OutputFormat::Json);
        assert_eq!(options.source_endpoint, "https://example.com/api/v3");
    }

    #[test]
    fn canister_info_options_require_a_canister_id() {
        let error = parse_test_options(info_command(), &[], CanisterInfoOptions::from_matches)
            .expect_err("missing canister id");

        assert!(matches!(error, IcCommandError::Usage(message) if message.contains("required")));
    }

    #[test]
    fn canister_count_options_preserve_official_filters() {
        let options = parse_test_options(
            count_command(),
            &[
                "--has-name",
                "true",
                "--subnet-id",
                "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe",
                "--language",
                "rust",
                "--language",
                "motoko",
                "--canister-type",
                "ledger",
                "--query",
                "ICP Ledger",
            ],
            CanisterCollectionOptions::from_matches,
        )
        .expect("count options");

        assert_eq!(options.filters.has_name, Some(true));
        assert_eq!(options.filters.languages, ["rust", "motoko"]);
        assert_eq!(options.filters.canister_types, ["ledger"]);
        assert_eq!(options.filters.query.as_deref(), Some("ICP Ledger"));
        assert_eq!(
            options.source_endpoint,
            DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT
        );
    }

    #[test]
    fn canister_page_options_are_bounded_and_cursors_are_exclusive() {
        let options = parse_test_options(
            page_command(),
            &["--limit", "100", "--after", CANISTER_ID],
            CanisterPageOptions::from_matches,
        )
        .expect("page options");

        assert_eq!(options.limit, MAX_IC_CANISTER_PAGE_LIMIT);
        assert_eq!(options.after.as_deref(), Some(CANISTER_ID));
        assert_eq!(options.before, None);

        let excessive = parse_test_options(
            page_command(),
            &["--limit", "101"],
            CanisterPageOptions::from_matches,
        )
        .expect_err("page limit above API maximum must fail");
        assert!(matches!(excessive, IcCommandError::Usage(_)));

        let conflicting = parse_test_options(
            page_command(),
            &["--after", CANISTER_ID, "--before", CANISTER_ID],
            CanisterPageOptions::from_matches,
        )
        .expect_err("page cursors must be exclusive");
        assert!(matches!(conflicting, IcCommandError::Usage(_)));
    }

    #[test]
    fn cli_and_library_page_defaults_remain_aligned() {
        let options = parse_test_options(page_command(), &[], CanisterPageOptions::from_matches)
            .expect("default page options");

        assert_eq!(options.limit, ic_query::ic::DEFAULT_IC_CANISTER_PAGE_LIMIT);
    }

    #[test]
    fn invalid_principal_fails_before_endpoint_or_network_use() {
        let error = crate::run([
            OsString::from("ic"),
            OsString::from("canister"),
            OsString::from("info"),
            OsString::from("not a principal"),
            OsString::from("--source-endpoint"),
            OsString::from("not a URL"),
        ])
        .expect_err("invalid principal must fail");

        assert!(matches!(
            error,
            crate::IcqCliError::Ic(IcCommandError::Host(IcHostError::InvalidPrincipal {
                field: "canister_id",
                ..
            }))
        ));
    }
}
