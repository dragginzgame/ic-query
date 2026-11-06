//! Module: nns::operational_status
//!
//! Responsibility: shared CLI parsing and dispatch for observed node-status views.
//! Does not own: Dashboard collection, cache policy, report projection, or rendering.
//! Boundary: adds the same status operation beneath node, Subnet, and provider nouns.

use crate::cli::clap::{parsed_options, with_options};
use crate::cli::common::OutputFormat;
use crate::storage::cache_root;
use crate::{
    cli::common::{
        COLLECTION_MODE_CACHE_REFRESH_STALE, collection_help, current_unix_secs, write_text_or_json,
    },
    nns::NnsCommandError,
    progress::StderrQueryProgress,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT;
use ic_query::ic::IcNodeStatusReadRequest;
use ic_query::ic::IcNodeStatusView;
use ic_query::ic::build_ic_node_provider_status_report;
use ic_query::ic::build_ic_node_status_report;
use ic_query::ic::build_ic_subnet_status_report;
use ic_query_text::ic::ic_node_provider_status_report_text;
use ic_query_text::ic::ic_node_status_report_text;
use ic_query_text::ic::ic_subnet_status_report_text;

use std::path::PathBuf;

const ALL_ARG: &str = "all";
const REFRESH_ARG: &str = "refresh";
const TARGET_ARG: &str = "target";

///
/// OperationalStatusSubject
///
/// NNS command noun selecting one projection over the shared node snapshot.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::nns) enum OperationalStatusSubject {
    Node,
    NodeProvider,
    Subnet,
}

impl OperationalStatusSubject {
    const fn command_name(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::NodeProvider => "node-provider",
            Self::Subnet => "subnet",
        }
    }

    const fn value_name(self) -> &'static str {
        match self {
            Self::Node => "node|node-prefix",
            Self::NodeProvider => "node-provider|node-provider-prefix",
            Self::Subnet => "subnet|subnet-prefix",
        }
    }

    const fn about(self) -> &'static str {
        match self {
            Self::Node => "Show observed operational status for IC nodes",
            Self::NodeProvider => "Show observed operational status grouped by node provider",
            Self::Subnet => "Show observed operational status and fault-distance by Subnet",
        }
    }

    const fn target_help(self) -> &'static str {
        match self {
            Self::Node => "Optional node principal or unique node-principal prefix",
            Self::NodeProvider => {
                "Optional node-provider principal or unique node-provider principal prefix"
            }
            Self::Subnet => "Optional Subnet principal or unique Subnet-principal prefix",
        }
    }
}

/// Build the shared `status` operation beneath one NNS command noun.
///
/// StatusOptions
///
/// Typed arguments for the command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct StatusOptions {
    #[arg(id = TARGET_ARG, id = TARGET_ARG, )]
    pub(super) target: Option<String>,
    #[arg(id = ALL_ARG, long = ALL_ARG, help = "Include fully-up rows or groups instead of attention rows only")]
    pub(super) all: bool,
    #[arg(id = REFRESH_ARG, long = REFRESH_ARG, help = "Force a live snapshot refresh before rendering this view")]
    pub(super) refresh: bool,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

pub(in crate::nns) fn command(subject: OperationalStatusSubject) -> ClapCommand {
    let command_name = subject.command_name();

    with_options::<StatusOptions>(ClapCommand::new("status")

        .about(subject.about())





        .after_help(collection_help(
            COLLECTION_MODE_CACHE_REFRESH_STALE,
            &format!(
                "Examples:\n  icq nns {command_name} status\n  icq nns {command_name} status <target-or-prefix> --json\n  icq nns {command_name} status --all --refresh"
            ),
        )))
        .mut_arg(TARGET_ARG, |arg| arg.value_name(subject.value_name()).help(subject.target_help()))
        .mut_arg("source-endpoint", |arg| arg.default_value(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API endpoint used for snapshot refreshes"))
}

/// Run one observed status projection over the shared cache identity.
pub(in crate::nns) async fn run(
    subject: OperationalStatusSubject,
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let (format, request) = status_request(matches, network, cache_root()?, current_unix_secs()?);
    let mut progress = StderrQueryProgress::new();

    match subject {
        OperationalStatusSubject::Node => {
            let report = build_ic_node_status_report(&request, &mut progress).await?;
            write_text_or_json(format, &report, ic_node_status_report_text)
        }
        OperationalStatusSubject::NodeProvider => {
            let report = build_ic_node_provider_status_report(&request, &mut progress).await?;
            write_text_or_json(format, &report, ic_node_provider_status_report_text)
        }
        OperationalStatusSubject::Subnet => {
            let report = build_ic_subnet_status_report(&request, &mut progress).await?;
            write_text_or_json(format, &report, ic_subnet_status_report_text)
        }
    }
}

fn status_request(
    matches: &ArgMatches,
    network: &str,
    cache_root: PathBuf,
    now_unix_secs: u64,
) -> (OutputFormat, IcNodeStatusReadRequest) {
    let options: StatusOptions = parsed_options(matches);
    let source = options.source;
    let format = source.format();
    let mut request =
        IcNodeStatusReadRequest::new(cache_root, network, source.source_endpoint, now_unix_secs);
    request.view = IcNodeStatusView {
        target: options.target,
        include_all: options.all,
    };
    request.force_refresh = options.refresh;

    (format, request)
}

#[cfg(test)]
mod tests;
