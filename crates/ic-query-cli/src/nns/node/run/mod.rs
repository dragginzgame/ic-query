mod info;
mod list;
mod refresh;

use crate::{
    nns::{NnsCommandError, command_cache_root},
    progress::announce_missing_mainnet_cache,
};
use clap::ArgMatches;
use ic_query::nns::{NnsInventoryCacheRequest, node::nns_node_cache_path};

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("status", matches)) => crate::nns::operational_status::run(
            crate::nns::operational_status::OperationalStatusSubject::Node,
            matches,
            network,
        ),
        Some(("list", matches)) => list::run_node_list(matches, network),
        Some(("info", matches)) => info::run_node_info(matches, network),
        Some(("refresh", matches)) => refresh::run_node_refresh(matches, network),
        _ => unreachable!("clap requires a known NNS node subcommand"),
    }
}

fn cache_request(network: &str) -> Result<NnsInventoryCacheRequest, NnsCommandError> {
    Ok(NnsInventoryCacheRequest::new(
        command_cache_root()?,
        network,
    ))
}

fn announce_missing_node_cache(cache: &NnsInventoryCacheRequest, source_endpoint: &str) {
    let path = nns_node_cache_path(&cache.cache_root, &cache.network);
    announce_missing_mainnet_cache(&cache.network, "node", &path, source_endpoint);
}
