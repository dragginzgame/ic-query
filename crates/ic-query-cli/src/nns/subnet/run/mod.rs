mod info;
mod list;
mod refresh;

use crate::nns::{NnsCommandError, command_cache_root};
use clap::ArgMatches;
use ic_query::subnet_catalog::SubnetCatalogCacheRequest;
pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("status", matches)) => crate::nns::operational_status::run(
            crate::nns::operational_status::OperationalStatusSubject::Subnet,
            matches,
            network,
        ),
        Some(("list", matches)) => list::run_catalog_list(matches, network),
        Some(("info", matches)) => info::run_catalog_info(matches, network),
        Some(("refresh", matches)) => refresh::run_catalog_refresh(matches, network),
        _ => unreachable!("clap requires a known NNS subnet subcommand"),
    }
}

fn cache_request(network: &str) -> Result<SubnetCatalogCacheRequest, NnsCommandError> {
    Ok(SubnetCatalogCacheRequest::new(
        command_cache_root()?,
        network,
    ))
}
