use crate::{
    cli::common::OutputFormat,
    nns::{
        NnsCommandError,
        leaf::NnsCommonOptions,
        parse_nns_matches,
        topology::{commands as topology_commands, report},
    },
};
use std::{ffi::OsString, path::PathBuf};

macro_rules! topology_read_options {
    ($name:ident, $request:path, $command:path, $usage:path) => {
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
                let matches = parse_nns_matches($command(), args, $usage)?;
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
                <$request>::new(icp_root, self.network, self.source_endpoint, now_unix_secs)
            }
        }
    };
}

pub(in crate::nns::topology) trait TopologyReadOptions<Request>:
    Sized
{
    fn parse_args(args: Vec<OsString>) -> Result<Self, NnsCommandError>;
    fn format(&self) -> OutputFormat;
    fn into_request(self, icp_root: PathBuf, now_unix_secs: u64) -> Request;
}

topology_read_options!(
    TopologySummaryOptions,
    report::NnsTopologySummaryRequest,
    topology_commands::topology_summary_command,
    topology_commands::topology_summary_usage
);
topology_read_options!(
    TopologyCoverageOptions,
    report::NnsTopologyCoverageRequest,
    topology_commands::topology_coverage_command,
    topology_commands::topology_coverage_usage
);
topology_read_options!(
    TopologyVersionsOptions,
    report::NnsTopologyVersionsRequest,
    topology_commands::topology_versions_command,
    topology_commands::topology_versions_usage
);
topology_read_options!(
    TopologyHealthOptions,
    report::NnsTopologyHealthRequest,
    topology_commands::topology_health_command,
    topology_commands::topology_health_usage
);
topology_read_options!(
    TopologyGapsOptions,
    report::NnsTopologyGapsRequest,
    topology_commands::topology_gaps_command,
    topology_commands::topology_gaps_usage
);
topology_read_options!(
    TopologyCapacityOptions,
    report::NnsTopologyCapacityRequest,
    topology_commands::topology_capacity_command,
    topology_commands::topology_capacity_usage
);
topology_read_options!(
    TopologyRegionsOptions,
    report::NnsTopologyRegionsRequest,
    topology_commands::topology_regions_command,
    topology_commands::topology_regions_usage
);
topology_read_options!(
    TopologyProvidersOptions,
    report::NnsTopologyProvidersRequest,
    topology_commands::topology_providers_command,
    topology_commands::topology_providers_usage
);
