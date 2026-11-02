mod list;

use crate::nns::{NnsCommandError, leaf};
use crate::storage::cache_root;
use clap::ArgMatches;
use ic_query::nns::NnsInventoryCacheRequest;
use ic_query::nns::node::build_nns_node_info_report_with_source_and_progress;
use ic_query::nns::node::refresh_nns_node_report;
use ic_query_text::nns::node::nns_node_info_report_text;

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("status", matches)) => {
            crate::nns::operational_status::run(
                crate::nns::operational_status::OperationalStatusSubject::Node,
                matches,
                network,
            )
            .await
        }
        Some(("list", matches)) => list::run_node_list(matches, network).await,
        Some(("info", matches)) => {
            leaf::run_cached_leaf_info(
                matches,
                network,
                async |request, progress| {
                    build_nns_node_info_report_with_source_and_progress(
                        request,
                        &ic_query::nns::LiveNnsSource::default(),
                        progress,
                    )
                    .await
                },
                &nns_node_info_report_text,
            )
            .await
        }
        Some(("refresh", matches)) => {
            leaf::run_cached_leaf_refresh(matches, network, refresh_nns_node_report).await
        }
        _ => unreachable!("clap requires a known NNS node subcommand"),
    }
}

fn cache_request(network: &str) -> Result<NnsInventoryCacheRequest, NnsCommandError> {
    Ok(NnsInventoryCacheRequest::new(cache_root()?, network))
}
