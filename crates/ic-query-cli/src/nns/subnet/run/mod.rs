mod info;
mod list;
mod refresh;

use crate::nns::NnsCommandError;
use crate::storage::cache_root;
use clap::ArgMatches;
use ic_query::subnet_catalog::SubnetCatalogCacheRequest;
pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("status", matches)) => {
            crate::nns::operational_status::run(
                crate::nns::operational_status::OperationalStatusSubject::Subnet,
                matches,
                network,
            )
            .await
        }
        Some(("list", matches)) => list::run_catalog_list(matches, network).await,
        Some(("info", matches)) => info::run_catalog_info(matches, network).await,
        Some(("refresh", matches)) => refresh::run_catalog_refresh(matches, network).await,
        _ => unreachable!("clap requires a known NNS subnet subcommand"),
    }
}

fn cache_request(network: &str) -> Result<SubnetCatalogCacheRequest, NnsCommandError> {
    Ok(SubnetCatalogCacheRequest::new(cache_root()?, network))
}
