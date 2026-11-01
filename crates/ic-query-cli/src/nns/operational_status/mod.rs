//! Module: nns::operational_status
//!
//! Responsibility: shared CLI parsing and dispatch for observed node-status views.
//! Does not own: Dashboard collection, cache policy, report projection, or rendering.
//! Boundary: adds the same status operation beneath node, Subnet, and provider nouns.

use crate::cli::common::OutputFormat;
use crate::storage::cache_root;
use crate::{
    cli::{
        clap::{flag_arg, string_option},
        common::{
            COLLECTION_MODE_CACHE_REFRESH_STALE, SourceOutputOptions, collection_help,
            current_unix_secs, json_arg, source_endpoint_arg, write_text_or_json,
        },
    },
    nns::NnsCommandError,
    progress::StderrQueryProgress,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcNodeStatusReadRequest, IcNodeStatusView,
    build_ic_node_provider_status_report, build_ic_node_status_report,
    build_ic_subnet_status_report, ic_node_provider_status_report_text, ic_node_status_report_text,
    ic_subnet_status_report_text,
};

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
pub(in crate::nns) fn command(subject: OperationalStatusSubject) -> ClapCommand {
    let command_name = subject.command_name();
    ClapCommand::new("status")
        .bin_name(format!("icq nns {command_name} status"))
        .about(subject.about())
        .arg(
            crate::cli::clap::value_arg(TARGET_ARG)
                .value_name(subject.value_name())
                .help(subject.target_help()),
        )
        .arg(
            flag_arg(ALL_ARG)
                .long(ALL_ARG)
                .help("Include fully-up rows or groups instead of attention rows only"),
        )
        .arg(
            flag_arg(REFRESH_ARG)
                .long(REFRESH_ARG)
                .help("Force a live snapshot refresh before rendering this view"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API endpoint used for snapshot refreshes"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_REFRESH_STALE,
            &format!(
                "Examples:\n  icq nns {command_name} status\n  icq nns {command_name} status <target-or-prefix> --json\n  icq nns {command_name} status --all --refresh"
            ),
        ))
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
    let source = SourceOutputOptions::from_matches(matches);
    let request =
        IcNodeStatusReadRequest::new(cache_root, network, source.source_endpoint, now_unix_secs)
            .with_view(IcNodeStatusView {
                target: string_option(matches, TARGET_ARG),
                include_all: matches.get_flag(ALL_ARG),
            })
            .with_force_refresh(matches.get_flag(REFRESH_ARG));
    (source.format, request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::{parse_matches, render_help};
    use std::ffi::OsString;

    #[test]
    fn every_status_subject_advertises_the_shared_options() {
        for subject in [
            OperationalStatusSubject::Node,
            OperationalStatusSubject::NodeProvider,
            OperationalStatusSubject::Subnet,
        ] {
            let help = render_help(command(subject));
            assert!(help.contains("--all"));
            assert!(help.contains("--json"));
            assert!(help.contains("--refresh"));
            assert!(help.contains("--source-endpoint"));
            assert!(help.contains("Cache-backed read"));
        }
    }

    #[test]
    fn status_options_parse_target_and_view_flags() {
        let matches = parse_matches(
            command(OperationalStatusSubject::Subnet),
            [
                OsString::from("tdb26"),
                OsString::from("--all"),
                OsString::from("--json"),
                OsString::from("--refresh"),
                OsString::from("--source-endpoint"),
                OsString::from("https://example.test/api/v3"),
            ],
        )
        .expect("parse status options");
        let (format, request) =
            status_request(&matches, "ic", PathBuf::from("/cache"), 1_780_531_200);

        assert_eq!(request.view.target.as_deref(), Some("tdb26"));
        assert_eq!(format, OutputFormat::Json);
        assert!(request.view.include_all);
        assert!(request.force_refresh);
        assert_eq!(
            request.refresh.source_endpoint,
            "https://example.test/api/v3"
        );
    }
}
