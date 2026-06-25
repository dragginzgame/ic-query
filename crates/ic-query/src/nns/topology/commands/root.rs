use crate::cli::clap::passthrough_subcommand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TopologyCommandHelp {
    name: &'static str,
    about: &'static str,
}

const TOPOLOGY_SUBCOMMANDS: &[TopologyCommandHelp] = &[
    TopologyCommandHelp {
        name: "summary",
        about: "Summarize cached mainnet NNS topology reports",
    },
    TopologyCommandHelp {
        name: "coverage",
        about: "Show cached mainnet NNS topology join coverage",
    },
    TopologyCommandHelp {
        name: "versions",
        about: "Show cached mainnet NNS topology component registry versions",
    },
    TopologyCommandHelp {
        name: "health",
        about: "Check cached mainnet NNS topology cache health",
    },
    TopologyCommandHelp {
        name: "gaps",
        about: "List cached mainnet NNS topology join gaps",
    },
    TopologyCommandHelp {
        name: "capacity",
        about: "Show cached mainnet NNS node-operator capacity",
    },
    TopologyCommandHelp {
        name: "regions",
        about: "Summarize cached mainnet NNS topology by region",
    },
    TopologyCommandHelp {
        name: "providers",
        about: "Summarize cached mainnet NNS topology by node provider",
    },
    TopologyCommandHelp {
        name: "refresh",
        about: "Refresh cached mainnet NNS topology component reports",
    },
];

pub(in crate::nns::topology) fn topology_command() -> clap::Command {
    TOPOLOGY_SUBCOMMANDS.iter().fold(
        clap::Command::new("topology")
            .bin_name("icq nns topology")
            .about("Inspect joined NNS topology metadata")
            .disable_help_flag(true),
        |command, subcommand| {
            command.subcommand(passthrough_subcommand(
                clap::Command::new(subcommand.name).about(subcommand.about),
            ))
        },
    )
}
