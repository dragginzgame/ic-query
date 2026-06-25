use super::{
    topology_capacity_command, topology_command, topology_coverage_command, topology_gaps_command,
    topology_health_command, topology_providers_command, topology_refresh_command,
    topology_regions_command, topology_summary_command, topology_versions_command,
};
use crate::cli::clap::render_help;

pub(in crate::nns) fn topology_usage() -> String {
    render_help(topology_command())
}

pub(in crate::nns) fn topology_summary_usage() -> String {
    render_help(topology_summary_command())
}

pub(in crate::nns) fn topology_coverage_usage() -> String {
    render_help(topology_coverage_command())
}

pub(in crate::nns) fn topology_versions_usage() -> String {
    render_help(topology_versions_command())
}

pub(in crate::nns) fn topology_health_usage() -> String {
    render_help(topology_health_command())
}

pub(in crate::nns) fn topology_gaps_usage() -> String {
    render_help(topology_gaps_command())
}

pub(in crate::nns) fn topology_capacity_usage() -> String {
    render_help(topology_capacity_command())
}

pub(in crate::nns) fn topology_regions_usage() -> String {
    render_help(topology_regions_command())
}

pub(in crate::nns) fn topology_providers_usage() -> String {
    render_help(topology_providers_command())
}

pub(in crate::nns) fn topology_refresh_usage() -> String {
    render_help(topology_refresh_command())
}
