//! Module: nns::operational_status
//!
//! Responsibility: shared CLI parsing and dispatch for observed node-status views.
//! Does not own: Dashboard collection, cache policy, report projection, or rendering.
//! Boundary: adds the same status operation beneath node, Subnet, and provider nouns.

use crate::{
    cli::{
        clap::{flag_arg, required_string, string_option},
        common::{
            COLLECTION_MODE_CACHE_REFRESH_STALE, OutputFormat, collection_help, json_arg,
            output_format, source_endpoint_arg, write_text_or_json,
        },
    },
    nns::{NnsCommandError, command_cache_root, now_unix_secs},
    progress::StderrQueryProgress,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcNodeStatusReadRequest, IcNodeStatusView,
    build_ic_node_provider_status_report, build_ic_node_status_report,
    build_ic_subnet_status_report, ic_node_provider_status_report_text, ic_node_status_report_text,
    ic_subnet_status_report_text,
};

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
pub(in crate::nns) fn run(
    subject: OperationalStatusSubject,
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    let options = OperationalStatusOptions::from_matches(matches);
    let view = IcNodeStatusView {
        target: options.target,
        include_all: options.include_all,
    };
    let request = IcNodeStatusReadRequest::new(
        command_cache_root()?,
        network,
        options.source_endpoint,
        now_unix_secs()?,
    )
    .with_view(view)
    .with_force_refresh(options.force_refresh);
    let mut progress = StderrQueryProgress::new();

    match subject {
        OperationalStatusSubject::Node => {
            let report = build_ic_node_status_report(&request, &mut progress)?;
            write_text_or_json(options.format, &report, ic_node_status_report_text)
        }
        OperationalStatusSubject::NodeProvider => {
            let report = build_ic_node_provider_status_report(&request, &mut progress)?;
            write_text_or_json(options.format, &report, ic_node_provider_status_report_text)
        }
        OperationalStatusSubject::Subnet => {
            let report = build_ic_subnet_status_report(&request, &mut progress)?;
            write_text_or_json(options.format, &report, ic_subnet_status_report_text)
        }
    }
}

struct OperationalStatusOptions {
    format: OutputFormat,
    target: Option<String>,
    include_all: bool,
    force_refresh: bool,
    source_endpoint: String,
}

impl OperationalStatusOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            format: output_format(matches),
            target: string_option(matches, TARGET_ARG),
            include_all: matches.get_flag(ALL_ARG),
            force_refresh: matches.get_flag(REFRESH_ARG),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
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
        let options = OperationalStatusOptions::from_matches(&matches);

        assert_eq!(options.target.as_deref(), Some("tdb26"));
        assert_eq!(options.format, OutputFormat::Json);
        assert!(options.include_all);
        assert!(options.force_refresh);
        assert_eq!(options.source_endpoint, "https://example.test/api/v3");
    }
}
