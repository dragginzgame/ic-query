//! Neuron read and cache dispatch.
mod cache;
mod neurons;
use super::{SnsCommandError, spec::sns_command};
pub fn command() -> clap::Command {
    sns_command()
}
pub async fn run_matches(matches: &clap::ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    match matches.subcommand() {
        Some(("neuron", matches)) => neurons::run_sns_neuron(matches, network).await,
        _ => unreachable!("clap requires the neuron command"),
    }
}
