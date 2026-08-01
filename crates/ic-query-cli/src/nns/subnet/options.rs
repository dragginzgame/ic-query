#[cfg(test)]
use super::commands::{info_command, list_command, refresh_command};
use crate::{
    cli::clap::{required_string, required_typed, typed_option},
    cli::common::output_format,
    nns::OutputFormat,
};
use clap::ArgMatches;
use ic_query::subnet_catalog::{ResolveAs, SubnetCatalogFilters};
#[cfg(test)]
use std::ffi::OsString;
use std::path::PathBuf;

///
/// CatalogListOptions
///
/// Parsed options accepted by `icq nns subnet list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct CatalogListOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) filters: SubnetCatalogFilters,
    pub(in crate::nns) show_ranges: bool,
    pub(in crate::nns) verbose: bool,
    pub(in crate::nns) range_limit: usize,
    pub(in crate::nns) range_offset: usize,
}

///
/// CatalogInfoOptions
///
/// Parsed options accepted by `icq nns subnet info`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct CatalogInfoOptions {
    pub(in crate::nns) input: String,
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) forced: Option<ResolveAs>,
}

///
/// CatalogRefreshOptions
///
/// Parsed options accepted by `icq nns subnet refresh`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct CatalogRefreshOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) lock_stale_after_seconds: u64,
    pub(in crate::nns) dry_run: bool,
    pub(in crate::nns) output_path: Option<PathBuf>,
}

impl CatalogListOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            network: network.to_string(),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
            filters: SubnetCatalogFilters {
                kind: typed_option(matches, "kind"),
                specialization: typed_option(matches, "specialization"),
                geographic_scope: typed_option(matches, "geo"),
            },
            show_ranges: matches.get_flag("show-ranges"),
            verbose: matches.get_flag("verbose"),
            range_limit: required_typed(matches, "range-limit"),
            range_offset: required_typed(matches, "range-offset"),
        }
    }

    #[cfg(test)]
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, crate::nns::NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::nns::parse_nns_matches(list_command(), args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}

impl CatalogInfoOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: network.to_string(),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
            forced: typed_option(matches, "as"),
        }
    }

    #[cfg(test)]
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, crate::nns::NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::nns::parse_nns_matches(info_command(), args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}

impl CatalogRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            network: network.to_string(),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
            lock_stale_after_seconds: required_typed(matches, "lock-stale-after"),
            dry_run: matches.get_flag("dry-run"),
            output_path: typed_option(matches, "output"),
        }
    }

    #[cfg(test)]
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, crate::nns::NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::nns::parse_nns_matches(refresh_command(), args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}
