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
mod proposals;
mod registry;
mod subnet;
#[cfg(test)]
mod tests;
mod topology;

use crate::{
    cli::{
        clap::{
            parse_matches_or_usage, parse_required_subcommand_or_usage, passthrough_subcommand,
            render_help,
        },
        common::{CurrentUnixSecsError, OutputFormat, current_unix_secs, write_text_or_json},
        help::collect_args_or_print_help_or_version,
    },
    storage::cache_root,
    version_text,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::{
    nns::{
        data_center::NnsDataCenterHostError, governance::NnsGovernanceHostError,
        neuron::NnsNeuronHostError, node::NnsNodeHostError,
        node_operator::NnsNodeOperatorHostError, node_provider::NnsNodeProviderHostError,
        proposals::NnsProposalHostError, registry::NnsRegistryHostError,
        topology::NnsTopologyHostError,
    },
    subnet_catalog::SubnetCatalogHostError,
};
use std::{ffi::OsString, io, path::PathBuf};
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
    #[error(transparent)]
    SubnetHost(#[from] SubnetCatalogHostError),
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
    Clock(#[from] CurrentUnixSecsError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let (command, args) = parse_nns_required_subcommand(nns_command(), args, usage)?;
    match command.as_str() {
        "subnet" => subnet::run(args),
        "data-center" => data_center::run(args),
        "governance" => governance::run(args),
        "node" => node::run(args),
        "neuron" => neuron::run(args),
        "node-provider" => node_provider::run(args),
        "node-operator" => node_operator::run(args),
        "proposal" => proposals::run(args),
        "registry" => registry::run(args),
        "topology" => topology::run(args),
        _ => unreachable!("NNS command only defines known subcommands"),
    }
}

pub(in crate::nns) fn command_args<I>(
    args: I,
    usage: impl FnOnce() -> String,
) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    collect_args_or_print_help_or_version(args, usage, version_text())
}

pub(in crate::nns) fn parse_nns_matches<I>(
    command: ClapCommand,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<ArgMatches, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    parse_matches_or_usage(command, args, usage).map_err(NnsCommandError::Usage)
}

pub(in crate::nns) fn parse_nns_required_subcommand<I>(
    command: ClapCommand,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<(String, Vec<OsString>), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    parse_required_subcommand_or_usage(command, args, usage).map_err(NnsCommandError::Usage)
}

fn now_unix_secs() -> Result<u64, NnsCommandError> {
    Ok(current_unix_secs()?)
}
fn command_cache_root() -> Result<PathBuf, NnsCommandError> {
    cache_root().map_err(|err| NnsCommandError::Usage(err.to_string()))
}

fn nns_command() -> ClapCommand {
    let families = [
        ("subnet", "Inspect and refresh NNS subnet metadata"),
        ("data-center", "Inspect NNS data-center metadata"),
        (
            "governance",
            "Inspect NNS Governance economics, metrics, and rewards",
        ),
        ("node", "Inspect NNS node metadata"),
        ("neuron", "Inspect public NNS Governance neuron views"),
        ("node-provider", "Inspect NNS node-provider metadata"),
        ("node-operator", "Inspect NNS node-operator metadata"),
        ("proposal", "Inspect NNS governance proposals"),
        ("registry", "Inspect NNS registry metadata"),
        ("topology", "Summarize joined NNS topology metadata"),
    ];
    families.into_iter().fold(
        ClapCommand::new("nns")
            .bin_name("icq nns")
            .about("Inspect NNS metadata")
            .disable_help_flag(true),
        |command, (name, about)| {
            command.subcommand(passthrough_subcommand(ClapCommand::new(name).about(about)))
        },
    )
}

fn usage() -> String {
    render_help(nns_command())
}
