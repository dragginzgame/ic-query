mod info;
mod list;
mod refresh;

use super::commands::{subnet_command, subnet_usage};
use crate::{
    nns::{NnsCommandError, command_args, command_icp_root, parse_nns_required_subcommand},
    subnet_catalog::SubnetCatalogCacheRequest,
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, subnet_usage) else {
        return Ok(());
    };
    let (command, args) = parse_nns_required_subcommand(subnet_command(), args, subnet_usage)?;

    match command.as_str() {
        "list" => list::run_catalog_list(args),
        "info" => info::run_catalog_info(args),
        "refresh" => refresh::run_catalog_refresh(args),
        _ => unreachable!("nns subnet dispatch command only defines known commands"),
    }
}

fn cache_request(network: &str) -> Result<SubnetCatalogCacheRequest, NnsCommandError> {
    let icp_root = command_icp_root()?;
    Ok(SubnetCatalogCacheRequest {
        icp_root,
        network: network.to_string(),
    })
}
