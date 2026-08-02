use super::{
    topology_capacity_command, topology_coverage_command, topology_gaps_command,
    topology_health_command, topology_providers_command, topology_refresh_command,
    topology_regions_command, topology_summary_command, topology_versions_command,
};

pub(in crate::nns) fn topology_command() -> clap::Command {
    clap::Command::new("topology")
        .bin_name("icq nns topology")
        .about("Inspect joined NNS topology metadata")
        .subcommand_required(true)
        .subcommand(topology_summary_command())
        .subcommand(topology_coverage_command())
        .subcommand(topology_versions_command())
        .subcommand(topology_health_command())
        .subcommand(topology_gaps_command())
        .subcommand(topology_capacity_command())
        .subcommand(topology_regions_command())
        .subcommand(topology_providers_command())
        .subcommand(topology_refresh_command())
}
