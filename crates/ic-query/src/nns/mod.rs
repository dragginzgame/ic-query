#[cfg(feature = "host")]
#[macro_use]
mod macros;
pub mod data_center;
#[cfg(feature = "host")]
mod leaf;
pub mod node;
pub mod node_operator;
pub mod node_provider;
pub mod proposals;
pub mod registry;
pub mod render;
#[cfg(feature = "cli")]
mod subnet;
pub mod topology;

#[cfg(all(test, feature = "cli"))]
mod tests;

#[cfg(feature = "cli")]
use crate::{
    cli::{
        clap::{
            parse_matches_or_usage, parse_required_subcommand_or_usage, passthrough_subcommand,
            render_help,
        },
        common::{CurrentUnixSecsError, OutputFormat, current_unix_secs, write_text_or_json},
        help::{collect_args_or_print_help_or_version, collect_args_or_print_help_or_version_flag},
    },
    nns::{
        data_center::report::NnsDataCenterHostError, node::report::NnsNodeHostError,
        node_operator::report::NnsNodeOperatorHostError,
        node_provider::report::NnsNodeProviderHostError, proposals::NnsProposalHostError,
        registry::report::NnsRegistryHostError, topology::report::NnsTopologyHostError,
    },
    project::icp_root as project_icp_root,
    subnet_catalog::SubnetCatalogHostError,
    version_text,
};
#[cfg(feature = "cli")]
use clap::{ArgMatches, Command as ClapCommand};
#[cfg(feature = "cli")]
use std::{ffi::OsString, io, path::PathBuf};
#[cfg(feature = "cli")]
use thiserror::Error as ThisError;

///
/// NnsCommandError
///
#[cfg(feature = "cli")]
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

#[cfg(feature = "cli")]
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
        "node" => node::run(args),
        "node-provider" => node_provider::run(args),
        "node-operator" => node_operator::run(args),
        "proposal" => proposals::run(args),
        "registry" => registry::run(args),
        "topology" => topology::run(args),
        _ => unreachable!("nns dispatch command only defines known commands"),
    }
}

#[cfg(feature = "cli")]
pub(in crate::nns) fn command_args<I>(
    args: I,
    usage: impl FnOnce() -> String,
) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    collect_args_or_print_help_or_version(args, usage, version_text())
}

#[cfg(feature = "cli")]
pub(in crate::nns) fn command_flag_args<I>(
    args: I,
    usage: impl FnOnce() -> String,
) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    collect_args_or_print_help_or_version_flag(args, usage, version_text())
}

#[cfg(feature = "cli")]
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

#[cfg(feature = "cli")]
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

#[cfg(feature = "cli")]
fn now_unix_secs() -> Result<u64, NnsCommandError> {
    Ok(current_unix_secs()?)
}

#[cfg(feature = "cli")]
fn command_icp_root() -> Result<PathBuf, NnsCommandError> {
    project_icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))
}

#[cfg(feature = "cli")]
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
            ClapCommand::new("proposal").about("Inspect NNS governance proposals"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("registry").about("Inspect NNS registry metadata"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("topology").about("Summarize joined NNS topology metadata"),
        ))
}

#[cfg(feature = "cli")]
fn usage() -> String {
    render_help(nns_command())
}
