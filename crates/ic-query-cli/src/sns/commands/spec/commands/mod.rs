//! Explicit Governance neuron command namespace.
mod args;
pub(in crate::sns::commands) mod neurons;
pub(in crate::sns::commands) fn sns_command() -> clap::Command {
    clap::Command::new("sns")
        .about("Inspect neuron ownership and stake")
        .subcommand(neurons::sns_neuron_command())
}
