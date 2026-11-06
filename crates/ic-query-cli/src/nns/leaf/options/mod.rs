//! Typed shared NNS arguments and their explicit network/storage context.

use crate::cli::{
    clap::parsed_options,
    common::{OutputFormat, SourceOutputOptions},
};
use clap::{ArgMatches, Args, builder::RangedU64ValueParser};
use ic_query::nns::{NnsGovernanceRefreshRequest, governance::NnsGovernanceRequest};
use std::path::PathBuf;

///
/// NnsCommonOptions
///
/// Source and output arguments plus the caller-selected network identity.
///

#[derive(Args, Clone, Debug, Eq, PartialEq)]
#[group(skip)]
pub(in crate::nns) struct NnsCommonOptions {
    #[arg(skip)]
    pub(in crate::nns) network: String,
    #[command(flatten)]
    pub(in crate::nns) source: SourceOutputOptions,
}

impl NnsCommonOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self::from_source(parsed_options(matches), network)
    }

    pub(in crate::nns) fn from_source(source: SourceOutputOptions, network: &str) -> Self {
        Self {
            network: network.to_string(),
            source,
        }
    }

    pub(in crate::nns) const fn format(&self) -> OutputFormat {
        self.source.format()
    }

    pub(in crate::nns) fn into_governance_request(
        self,
        now: u64,
    ) -> (OutputFormat, NnsGovernanceRequest) {
        (
            self.format(),
            NnsGovernanceRequest::replica_query_from_unix_secs(
                self.network,
                self.source.source_endpoint,
                now,
                "ic-query",
            ),
        )
    }
}

///
/// NnsLeafListOptions
///
/// Common inventory-list arguments; family specs supply descriptive help.
///

#[derive(Args, Clone, Debug, Eq, PartialEq)]
#[group(skip)]
pub(in crate::nns) struct NnsLeafListOptions {
    #[command(flatten)]
    pub(in crate::nns) common: NnsCommonOptions,
    #[arg(long)]
    pub(in crate::nns) verbose: bool,
}

impl NnsLeafListOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let mut options: Self = parsed_options(matches);
        options.common.network = network.to_string();
        options
    }
}

///
/// NnsLeafInfoOptions
///
/// One inventory identifier with the shared acquisition arguments.
///

#[derive(Args, Clone, Debug, Eq, PartialEq)]
#[group(skip)]
pub(in crate::nns) struct NnsLeafInfoOptions {
    pub(in crate::nns) input: String,
    #[command(flatten)]
    pub(in crate::nns) common: NnsCommonOptions,
}

impl NnsLeafInfoOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let mut options: Self = parsed_options(matches);
        options.common.network = network.to_string();
        options
    }
}

///
/// NnsLeafRefreshOptions
///
/// Shared inventory refresh controls, preserving native output-path bytes.
///

#[derive(Args, Clone, Debug, Eq, PartialEq)]
#[group(skip)]
pub(in crate::nns) struct NnsLeafRefreshOptions {
    #[command(flatten)]
    pub(in crate::nns) common: NnsCommonOptions,
    #[arg(id = "dry-run", long)]
    pub(in crate::nns) dry_run: bool,
    #[arg(id = "output", long = "output", value_name = "path")]
    pub(in crate::nns) output_path: Option<PathBuf>,
}

impl NnsLeafRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let mut options: Self = parsed_options(matches);
        options.common.network = network.to_string();
        options
    }
}

///
/// NnsPagedRefreshOptions
///
/// Common paged refresh arguments; each family supplies its page-size policy.
///

#[derive(Args, Clone, Debug, Eq, PartialEq)]
#[group(skip)]
pub(in crate::nns) struct NnsPagedRefreshOptions {
    #[command(flatten)]
    pub(in crate::nns) common: NnsCommonOptions,
    #[arg(id = "page-size", long, value_name = "count", default_value = "100", value_parser = RangedU64ValueParser::<u32>::new().range(1..))]
    pub(in crate::nns) page_size: u32,
    #[arg(id = "max-pages", long, value_name = "count", value_parser = RangedU64ValueParser::<u32>::new().range(1..))]
    pub(in crate::nns) max_pages: Option<u32>,
}

impl NnsPagedRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let mut options: Self = parsed_options(matches);
        options.common.network = network.to_string();
        options
    }

    pub(in crate::nns) fn into_request(
        self,
        cache_root: PathBuf,
        now: u64,
    ) -> NnsGovernanceRefreshRequest {
        let mut request = NnsGovernanceRefreshRequest::new(
            cache_root,
            self.common.network,
            self.common.source.source_endpoint,
            now,
            self.page_size,
        );
        request.max_pages = self.max_pages;
        request
    }
}
