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
    ($name:ident, $command:path, $usage:path) => {
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

        impl TopologyReadOptions for $name {
            fn parse_args(args: Vec<OsString>) -> Result<Self, NnsCommandError> {
                Self::parse(args)
            }

            fn format(&self) -> OutputFormat {
                self.format
            }

            fn into_request(
                self,
                icp_root: PathBuf,
                now_unix_secs: u64,
            ) -> report::NnsTopologyReadRequest {
                report::NnsTopologyReadRequest::new(
                    icp_root,
                    self.network,
                    self.source_endpoint,
                    now_unix_secs,
                )
            }
        }
    };
}

pub(in crate::nns::topology) trait TopologyReadOptions: Sized {
    fn parse_args(args: Vec<OsString>) -> Result<Self, NnsCommandError>;
    fn format(&self) -> OutputFormat;
    fn into_request(self, icp_root: PathBuf, now_unix_secs: u64) -> report::NnsTopologyReadRequest;
}

topology_read_options!(
    TopologySummaryOptions,
    topology_commands::topology_summary_command,
    topology_commands::topology_summary_usage
);
topology_read_options!(
    TopologyCoverageOptions,
    topology_commands::topology_coverage_command,
    topology_commands::topology_coverage_usage
);
topology_read_options!(
    TopologyVersionsOptions,
    topology_commands::topology_versions_command,
    topology_commands::topology_versions_usage
);
topology_read_options!(
    TopologyHealthOptions,
    topology_commands::topology_health_command,
    topology_commands::topology_health_usage
);
topology_read_options!(
    TopologyGapsOptions,
    topology_commands::topology_gaps_command,
    topology_commands::topology_gaps_usage
);
topology_read_options!(
    TopologyCapacityOptions,
    topology_commands::topology_capacity_command,
    topology_commands::topology_capacity_usage
);
topology_read_options!(
    TopologyRegionsOptions,
    topology_commands::topology_regions_command,
    topology_commands::topology_regions_usage
);
topology_read_options!(
    TopologyProvidersOptions,
    topology_commands::topology_providers_command,
    topology_commands::topology_providers_usage
);
