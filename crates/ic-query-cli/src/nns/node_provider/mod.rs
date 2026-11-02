//! Module: nns::node_provider
//!
//! Responsibility: assemble node-provider CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-provider arguments to the typed library API.

use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;
use ic_query::nns::node_provider::{
    build_nns_node_provider_info_report_with_source_and_progress,
    build_nns_node_provider_list_report_with_source_and_progress,
    nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text, refresh_nns_node_provider_report,
};

mod reward;
mod spec;

pub(in crate::nns) fn command() -> clap::Command {
    leaf::command(
        &spec::NODE_PROVIDER_SPEC,
        ic_query::nns::node_provider::DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
    )
    .subcommand(crate::nns::operational_status::command(
        crate::nns::operational_status::OperationalStatusSubject::NodeProvider,
    ))
    .subcommand(reward::command())
}

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("reward", matches)) => reward::run(matches, network).await,
        Some(("status", matches)) => {
            crate::nns::operational_status::run(
                crate::nns::operational_status::OperationalStatusSubject::NodeProvider,
                matches,
                network,
            )
            .await
        }
        Some(("list", matches)) => {
            leaf::run_cached_leaf_list(
                matches,
                network,
                async |request, progress| {
                    build_nns_node_provider_list_report_with_source_and_progress(
                        request,
                        &ic_query::nns::LiveNnsSource::default(),
                        progress,
                    )
                    .await
                },
                &nns_node_provider_list_report_text,
                &nns_node_provider_list_report_verbose_text,
            )
            .await
        }
        Some(("info", matches)) => {
            leaf::run_cached_leaf_info(
                matches,
                network,
                async |request, progress| {
                    build_nns_node_provider_info_report_with_source_and_progress(
                        request,
                        &ic_query::nns::LiveNnsSource::default(),
                        progress,
                    )
                    .await
                },
                &nns_node_provider_info_report_text,
            )
            .await
        }
        Some(("refresh", matches)) => {
            leaf::run_cached_leaf_refresh(matches, network, refresh_nns_node_provider_report).await
        }
        _ => unreachable!("clap requires a known NNS leaf subcommand"),
    }
}

#[cfg(test)]
pub(in crate::nns) use spec::NODE_PROVIDER_SPEC;
