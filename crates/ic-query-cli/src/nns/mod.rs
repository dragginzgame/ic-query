//! NNS command-line parsing and dispatch.

#[macro_use]
mod macros;
mod data_center;
mod governance;
mod leaf;
mod neuron;
mod node;
mod node_operator;
mod node_provider;
mod operational_status;
mod proposals;
mod registry;
mod subnet;
#[cfg(test)]
mod tests;
mod topology;

use crate::{
    cli::common::{CurrentUnixSecsError, OutputFormat, current_unix_secs, write_text_or_json},
    storage::cache_root,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::{
    ic::IcNodeStatusHostError,
    nns::{
        data_center::NnsDataCenterHostError, governance::NnsGovernanceHostError,
        neuron::NnsNeuronHostError, node::NnsNodeHostError,
        node_operator::NnsNodeOperatorHostError, node_provider::NnsNodeProviderHostError,
        proposals::NnsProposalHostError, registry::NnsRegistryHostError,
        topology::NnsTopologyHostError,
    },
    subnet_catalog::{SubnetCatalogHostError, SubnetCatalogRemediation},
};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// NnsCommandError
///
/// Errors surfaced while parsing or running an NNS command.
///

#[derive(Debug, ThisError)]
pub enum NnsCommandError {
    #[error("{0}")]
    Usage(String),
    #[error("{message}")]
    SubnetHost {
        message: String,
        #[source]
        source: SubnetCatalogHostError,
    },
    #[error(transparent)]
    DataCenterHost(#[from] NnsDataCenterHostError),
    #[error(transparent)]
    GovernanceHost(#[from] NnsGovernanceHostError),
    #[error(transparent)]
    NodeHost(#[from] NnsNodeHostError),
    #[error(transparent)]
    NeuronHost(#[from] NnsNeuronHostError),
    #[error(transparent)]
    NodeProviderHost(#[from] NnsNodeProviderHostError),
    #[error(transparent)]
    NodeOperatorHost(#[from] NnsNodeOperatorHostError),
    #[error(transparent)]
    ProposalHost(#[from] NnsProposalHostError),
    #[error(transparent)]
    RegistryHost(#[from] NnsRegistryHostError),
    #[error(transparent)]
    TopologyHost(#[from] NnsTopologyHostError),
    #[error(transparent)]
    NodeStatusHost(#[from] IcNodeStatusHostError),
    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<SubnetCatalogHostError> for NnsCommandError {
    fn from(source: SubnetCatalogHostError) -> Self {
        let mut message = source.to_string();
        match source.remediation() {
            Some(SubnetCatalogRemediation::UseMainnet) => {
                message.push_str("\n\nTry:\n  icq --network ic nns subnet list");
            }
            Some(SubnetCatalogRemediation::RefreshCatalog) => {
                message.push_str("\n\nTry:\n  icq nns subnet refresh");
            }
            None => {}
        }
        Self::SubnetHost { message, source }
    }
}

pub fn run_matches(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("subnet", matches)) => subnet::run(matches, network),
        Some(("data-center", matches)) => data_center::run(matches, network),
        Some(("governance", matches)) => governance::run(matches, network),
        Some(("node", matches)) => node::run(matches, network),
        Some(("neuron", matches)) => neuron::run(matches, network),
        Some(("node-provider", matches)) => node_provider::run(matches, network),
        Some(("node-operator", matches)) => node_operator::run(matches, network),
        Some(("proposal", matches)) => proposals::run(matches, network),
        Some(("registry", matches)) => registry::run(matches, network),
        Some(("topology", matches)) => topology::run(matches, network),
        _ => unreachable!("clap requires a known NNS subcommand"),
    }
}

fn now_unix_secs() -> Result<u64, NnsCommandError> {
    Ok(current_unix_secs()?)
}
fn command_cache_root() -> Result<PathBuf, NnsCommandError> {
    cache_root().map_err(|err| NnsCommandError::Usage(err.to_string()))
}

pub fn command() -> ClapCommand {
    ClapCommand::new("nns")
        .bin_name("icq nns")
        .about("Inspect NNS metadata")
        .subcommand(subnet::command())
        .subcommand(data_center::command())
        .subcommand(governance::command())
        .subcommand(node::command())
        .subcommand(neuron::command())
        .subcommand(node_provider::command())
        .subcommand(node_operator::command())
        .subcommand(proposals::command())
        .subcommand(registry::command())
        .subcommand(topology::command())
}
