#[macro_use]
mod macros;
mod data_center;
mod leaf;
mod node;
mod node_operator;
mod node_provider;
mod registry;
pub mod render;
mod subnet;
mod topology;

#[cfg(test)]
mod tests;

use crate::{
    cli::{
        clap::{parse_required_subcommand, passthrough_subcommand, render_help},
        common::{OutputFormat, current_unix_secs, write_text_or_json},
        help::print_help_or_version,
    },
    nns::{
        data_center::report::NnsDataCenterHostError, node::report::NnsNodeHostError,
        node_operator::report::NnsNodeOperatorHostError,
        node_provider::report::NnsNodeProviderHostError, registry::report::NnsRegistryHostError,
        topology::report::NnsTopologyHostError,
    },
    subnet_catalog::SubnetCatalogHostError,
    version_text,
};
use clap::Command as ClapCommand;
use std::{ffi::OsString, io};
use thiserror::Error as ThisError;

///
/// NnsCommandError
///
#[derive(Debug, ThisError)]
pub enum NnsCommandError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    SubnetHost(#[from] SubnetCatalogHostError),

    #[error(transparent)]
    DataCenterHost(#[from] NnsDataCenterHostError),

    #[error(transparent)]
    NodeHost(#[from] NnsNodeHostError),

    #[error(transparent)]
    NodeProviderHost(#[from] NnsNodeProviderHostError),

    #[error(transparent)]
    NodeOperatorHost(#[from] NnsNodeOperatorHostError),

    #[error(transparent)]
    RegistryHost(#[from] NnsRegistryHostError),

    #[error(transparent)]
    TopologyHost(#[from] NnsTopologyHostError),

    #[error("system clock before unix epoch: {0}")]
    Clock(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(nns_command(), args)
        .map_err(|_| NnsCommandError::Usage(usage()))?;

    match command.as_str() {
        "subnet" => subnet::run(args),
        "data-center" => data_center::run(args),
        "node" => node::run(args),
        "node-provider" => node_provider::run(args),
        "node-operator" => node_operator::run(args),
        "registry" => registry::run(args),
        "topology" => topology::run(args),
        _ => unreachable!("nns dispatch command only defines known commands"),
    }
}

fn now_unix_secs() -> Result<u64, NnsCommandError> {
    current_unix_secs().map_err(NnsCommandError::Clock)
}

fn nns_command() -> ClapCommand {
    ClapCommand::new("nns")
        .bin_name("icq nns")
        .about("Inspect NNS metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("subnet").about("Inspect and refresh NNS subnet metadata"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("data-center").about("Inspect NNS data-center metadata"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("node").about("Inspect NNS node metadata"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("node-provider").about("Inspect NNS node-provider metadata"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("node-operator").about("Inspect NNS node-operator metadata"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("registry").about("Inspect NNS registry metadata"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("topology").about("Summarize joined NNS topology metadata"),
        ))
}

fn usage() -> String {
    render_help(nns_command())
}
