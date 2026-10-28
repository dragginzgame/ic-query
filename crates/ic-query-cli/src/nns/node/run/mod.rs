mod list;

use crate::nns::{NnsCommandError, leaf};
use crate::storage::cache_root;
use clap::ArgMatches;
use ic_query::nns::NnsInventoryCacheRequest;
use ic_query::nns::node::{
    build_nns_node_info_report_with_progress, nns_node_info_report_text, refresh_nns_node_report,
};

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
                build_nns_node_info_report_with_progress,
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
