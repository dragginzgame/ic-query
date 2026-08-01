mod info;
mod list;
mod refresh;

use super::commands::{subnet_command, subnet_usage};
use crate::{
    nns::{NnsCommandError, command_args, command_cache_root, parse_nns_required_subcommand},
    progress::announce_missing_mainnet_cache,
};
use ic_query::subnet_catalog::{SubnetCatalogCacheRequest, subnet_catalog_path};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, subnet_usage) else {
        return Ok(());
    };
    let (command, args) = parse_nns_required_subcommand(subnet_command(), args)?;

    match command.as_str() {
        "list" => list::run_catalog_list(args),
        "info" => info::run_catalog_info(args),
        "refresh" => refresh::run_catalog_refresh(args),
        _ => unreachable!("nns subnet dispatch command only defines known commands"),
    }
}

fn cache_request(network: &str) -> Result<SubnetCatalogCacheRequest, NnsCommandError> {
    Ok(SubnetCatalogCacheRequest::new(
        command_cache_root()?,
        network,
    ))
}

fn announce_missing_catalog(cache: &SubnetCatalogCacheRequest, source_endpoint: &str) {
    let path = subnet_catalog_path(&cache.cache_root, &cache.network);
    announce_missing_mainnet_cache(&cache.network, "subnet catalog", &path, source_endpoint);
}
