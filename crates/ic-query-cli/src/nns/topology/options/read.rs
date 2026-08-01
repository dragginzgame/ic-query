#[cfg(test)]
use crate::nns::topology::commands as topology_commands;
use crate::{cli::common::OutputFormat, nns::leaf::NnsCommonOptions};
use clap::ArgMatches;
use ic_query::nns::topology::NnsTopologyReadRequest;
#[cfg(test)]
use std::ffi::OsString;
use std::path::PathBuf;

macro_rules! topology_read_options {
    ($name:ident, $command:path) => {
        #[doc = ""]
        #[doc = stringify!($name)]
        #[doc = ""]
        #[doc = "Parsed options accepted by an NNS topology read command."]
        #[doc = ""]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(in crate::nns) struct $name {
            pub(in crate::nns) network: String,
            pub(in crate::nns) format: OutputFormat,
            pub(in crate::nns) source_endpoint: String,
        }

        impl $name {
            #[cfg(test)]
            pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, crate::nns::NnsCommandError>
            where
                I: IntoIterator<Item = OsString>,
            {
                let matches = crate::nns::parse_nns_matches($command(), args)?;
                Ok(<Self as TopologyReadOptions>::from_matches(
                    &matches,
                    ic_query::subnet_catalog::MAINNET_NETWORK,
                ))
            }
        }

        impl TopologyReadOptions for $name {
            fn from_matches(matches: &ArgMatches, network: &str) -> Self {
                let common = NnsCommonOptions::from_matches(matches, network);
                Self {
                    network: common.network,
                    format: common.format,
                    source_endpoint: common.source_endpoint,
                }
            }

            fn format(&self) -> OutputFormat {
                self.format
            }

            fn into_request(
                self,
                cache_root: PathBuf,
                now_unix_secs: u64,
            ) -> NnsTopologyReadRequest {
                NnsTopologyReadRequest::new(
                    cache_root,
                    self.network,
                    self.source_endpoint,
                    now_unix_secs,
                )
            }
        }
    };
}

///
/// TopologyReadOptions
///
/// Request conversion shared by NNS topology read command variants.
///

pub(in crate::nns::topology) trait TopologyReadOptions: Sized {
    fn from_matches(matches: &ArgMatches, network: &str) -> Self;
    fn format(&self) -> OutputFormat;
    fn into_request(self, cache_root: PathBuf, now_unix_secs: u64) -> NnsTopologyReadRequest;
}

topology_read_options!(
    TopologySummaryOptions,
    topology_commands::topology_summary_command
);
topology_read_options!(
    TopologyCoverageOptions,
    topology_commands::topology_coverage_command
);
topology_read_options!(
    TopologyVersionsOptions,
    topology_commands::topology_versions_command
);
topology_read_options!(
    TopologyHealthOptions,
    topology_commands::topology_health_command
);
topology_read_options!(
    TopologyGapsOptions,
    topology_commands::topology_gaps_command
);
topology_read_options!(
    TopologyCapacityOptions,
    topology_commands::topology_capacity_command
);
topology_read_options!(
    TopologyRegionsOptions,
    topology_commands::topology_regions_command
);
topology_read_options!(
    TopologyProvidersOptions,
    topology_commands::topology_providers_command
);
