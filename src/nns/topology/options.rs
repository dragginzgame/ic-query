use super::commands::{
    DRY_RUN_ARG, LOCK_STALE_AFTER_ARG, topology_capacity_command, topology_capacity_usage,
    topology_coverage_command, topology_coverage_usage, topology_gaps_command, topology_gaps_usage,
    topology_health_command, topology_health_usage, topology_providers_command,
    topology_providers_usage, topology_refresh_command, topology_refresh_usage,
    topology_regions_command, topology_regions_usage, topology_summary_command,
    topology_summary_usage, topology_versions_command, topology_versions_usage,
};
use super::report::{
    NnsTopologyCapacityRequest, NnsTopologyCoverageRequest, NnsTopologyGapsRequest,
    NnsTopologyHealthRequest, NnsTopologyProvidersRequest, NnsTopologyRegionsRequest,
    NnsTopologySummaryRequest, NnsTopologyVersionsRequest,
};
use crate::{
    cli::{clap::parse_matches, clap::required_typed, common::OutputFormat},
    nns::{NnsCommandError, leaf::NnsCommonOptions},
};
use std::{ffi::OsString, path::PathBuf};

macro_rules! topology_read_options {
    ($name:ident, $request:ident, $command:ident, $usage:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(in crate::nns) struct $name {
            pub(in crate::nns) network: String,
            pub(in crate::nns) format: OutputFormat,
            pub(in crate::nns) source_endpoint: String,
        }

        impl $name {
            pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
            where
                I: IntoIterator<Item = OsString>,
            {
                let matches = parse_matches($command(), args)
                    .map_err(|_| NnsCommandError::Usage($usage()))?;
                let common = NnsCommonOptions::from_matches(&matches);
                Ok(Self {
                    network: common.network,
                    format: common.format,
                    source_endpoint: common.source_endpoint,
                })
            }
        }

        impl TopologyReadOptions<$request> for $name {
            fn parse_args(args: Vec<OsString>) -> Result<Self, NnsCommandError> {
                Self::parse(args)
            }

            fn format(&self) -> OutputFormat {
                self.format
            }

            fn into_request(self, icp_root: PathBuf, now_unix_secs: u64) -> $request {
                $request {
                    icp_root,
                    network: self.network,
                    source_endpoint: self.source_endpoint,
                    now_unix_secs,
                }
            }
        }
    };
}

pub(super) trait TopologyReadOptions<Request>: Sized {
    fn parse_args(args: Vec<OsString>) -> Result<Self, NnsCommandError>;
    fn format(&self) -> OutputFormat;
    fn into_request(self, icp_root: PathBuf, now_unix_secs: u64) -> Request;
}

topology_read_options!(
    TopologySummaryOptions,
    NnsTopologySummaryRequest,
    topology_summary_command,
    topology_summary_usage
);
topology_read_options!(
    TopologyCoverageOptions,
    NnsTopologyCoverageRequest,
    topology_coverage_command,
    topology_coverage_usage
);
topology_read_options!(
    TopologyVersionsOptions,
    NnsTopologyVersionsRequest,
    topology_versions_command,
    topology_versions_usage
);
topology_read_options!(
    TopologyHealthOptions,
    NnsTopologyHealthRequest,
    topology_health_command,
    topology_health_usage
);
topology_read_options!(
    TopologyGapsOptions,
    NnsTopologyGapsRequest,
    topology_gaps_command,
    topology_gaps_usage
);
topology_read_options!(
    TopologyCapacityOptions,
    NnsTopologyCapacityRequest,
    topology_capacity_command,
    topology_capacity_usage
);
topology_read_options!(
    TopologyRegionsOptions,
    NnsTopologyRegionsRequest,
    topology_regions_command,
    topology_regions_usage
);
topology_read_options!(
    TopologyProvidersOptions,
    NnsTopologyProvidersRequest,
    topology_providers_command,
    topology_providers_usage
);

///
/// TopologyRefreshOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct TopologyRefreshOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) lock_stale_after_seconds: u64,
    pub(in crate::nns) dry_run: bool,
}

impl TopologyRefreshOptions {
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(topology_refresh_command(), args)
            .map_err(|_| NnsCommandError::Usage(topology_refresh_usage()))?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            lock_stale_after_seconds: required_typed(&matches, LOCK_STALE_AFTER_ARG),
            dry_run: matches.get_flag(DRY_RUN_ARG),
        })
    }
}
