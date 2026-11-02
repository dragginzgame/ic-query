//! Module: nns::node_operator
//!
//! Responsibility: assemble node-operator CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-operator arguments to the typed library API.

use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;
use ic_query::nns::node_operator::build_nns_node_operator_info_report_with_source_and_progress;
use ic_query::nns::node_operator::build_nns_node_operator_list_report_with_source_and_progress;
use ic_query::nns::node_operator::refresh_nns_node_operator_report;
use ic_query_text::nns::node_operator::nns_node_operator_info_report_text;
use ic_query_text::nns::node_operator::nns_node_operator_list_report_text;
use ic_query_text::nns::node_operator::nns_node_operator_list_report_verbose_text;

mod spec;

pub(in crate::nns) fn command() -> clap::Command {
    leaf::command(
        &spec::NODE_OPERATOR_SPEC,
        ic_query::nns::node_operator::DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => {
            leaf::run_cached_leaf_list(
                matches,
                network,
                async |request, progress| {
                    build_nns_node_operator_list_report_with_source_and_progress(
                        request,
                        &ic_query::nns::LiveNnsSource::default(),
                        progress,
                    )
                    .await
                },
                &nns_node_operator_list_report_text,
                &nns_node_operator_list_report_verbose_text,
            )
            .await
        }
        Some(("info", matches)) => {
            leaf::run_cached_leaf_info(
                matches,
                network,
                async |request, progress| {
                    build_nns_node_operator_info_report_with_source_and_progress(
                        request,
                        &ic_query::nns::LiveNnsSource::default(),
                        progress,
                    )
                    .await
                },
                &nns_node_operator_info_report_text,
            )
            .await
        }
        Some(("refresh", matches)) => {
            leaf::run_cached_leaf_refresh(matches, network, refresh_nns_node_operator_report).await
        }
        _ => unreachable!("clap requires a known NNS leaf subcommand"),
    }
}

#[cfg(test)]
pub(in crate::nns) use spec::NODE_OPERATOR_SPEC;
