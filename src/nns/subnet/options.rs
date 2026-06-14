use super::commands::{
    info_command, info_usage, list_command, list_usage, refresh_command, refresh_usage,
};
use crate::{
    cli::clap::{parse_matches, required_string, required_typed, typed_option},
    nns::{NnsCommandError, OutputFormat},
    subnet_catalog::{ResolveAs, SubnetCatalogFilters},
};
use std::{ffi::OsString, path::PathBuf};

///
/// CatalogListOptions
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
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(list_command(), args)
            .map_err(|_| NnsCommandError::Usage(list_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            filters: SubnetCatalogFilters {
                kind: typed_option(&matches, "kind"),
                specialization: typed_option(&matches, "specialization"),
                geographic_scope: typed_option(&matches, "geo"),
            },
            show_ranges: matches.get_flag("show-ranges"),
            verbose: matches.get_flag("verbose"),
            range_limit: required_typed(&matches, "range-limit"),
            range_offset: required_typed(&matches, "range-offset"),
        })
    }
}

impl CatalogInfoOptions {
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(info_command(), args)
            .map_err(|_| NnsCommandError::Usage(info_usage()))?;
        Ok(Self {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            forced: typed_option(&matches, "as"),
        })
    }
}

impl CatalogRefreshOptions {
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(refresh_command(), args)
            .map_err(|_| NnsCommandError::Usage(refresh_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            lock_stale_after_seconds: required_typed(&matches, "lock-stale-after"),
            dry_run: matches.get_flag("dry-run"),
            output_path: typed_option(&matches, "output"),
        })
    }
}
