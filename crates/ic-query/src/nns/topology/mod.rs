#[cfg(feature = "cli")]
mod commands;
#[cfg(feature = "cli")]
mod options;
pub mod report;
#[cfg(feature = "cli")]
mod run;

#[cfg(all(test, feature = "cli"))]
pub(super) use commands::{
    topology_capacity_usage, topology_coverage_usage, topology_gaps_usage, topology_health_usage,
    topology_providers_usage, topology_refresh_usage, topology_regions_usage,
    topology_summary_usage, topology_usage, topology_versions_usage,
};
#[cfg(all(test, feature = "cli"))]
pub(super) use options::{
    TopologyCapacityOptions, TopologyCoverageOptions, TopologyGapsOptions, TopologyHealthOptions,
    TopologyProvidersOptions, TopologyRefreshOptions, TopologyRegionsOptions,
    TopologySummaryOptions, TopologyVersionsOptions,
};
#[cfg(feature = "cli")]
pub(super) use run::run;
