mod info;
mod list;
mod refresh;

use super::commands::NODE_SPEC;
use crate::{
    nns::{NnsCommandError, command_cache_root, leaf},
    progress::announce_missing_mainnet_cache,
};
use ic_query::nns::{NnsInventoryCacheRequest, node::nns_node_cache_path};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_leaf(
        args,
        &NODE_SPEC,
        list::run_node_list,
        info::run_node_info,
        refresh::run_node_refresh,
    )
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
