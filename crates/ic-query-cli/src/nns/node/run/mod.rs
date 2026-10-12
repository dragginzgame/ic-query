mod info;
mod list;
mod refresh;

use crate::nns::{NnsCommandError, command_cache_root};
use clap::ArgMatches;
use ic_query::nns::NnsInventoryCacheRequest;

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
        Some(("info", matches)) => info::run_node_info(matches, network).await,
        Some(("refresh", matches)) => refresh::run_node_refresh(matches, network).await,
        _ => unreachable!("clap requires a known NNS node subcommand"),
    }
}

fn cache_request(network: &str) -> Result<NnsInventoryCacheRequest, NnsCommandError> {
    Ok(NnsInventoryCacheRequest::new(
        command_cache_root()?,
        network,
    ))
}
