//! Module: nns::leaf::options::common
//!
//! Responsibility: parse common clap options for generic NNS leaf commands.
//! Does not own: command specs, report requests, or output rendering.
//! Boundary: converts shared clap matches into reusable leaf option values.

use crate::cli::{
    clap::{required_string, required_typed, typed_option},
    common::{OutputFormat, SOURCE_ENDPOINT_ARG, output_format},
};
use clap::ArgMatches;
use ic_query::nns::NnsGovernanceRefreshRequest;
use std::path::PathBuf;

///
/// NnsCommonOptions
///
/// Common parsed options shared by generic NNS leaf command variants.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsCommonOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl NnsCommonOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            network: network.to_string(),
            format: output_format(matches),
            source_endpoint: required_string(matches, SOURCE_ENDPOINT_ARG),
        }
    }
}

///
/// NnsPagedRefreshOptions
///
/// Common endpoint and page bounds for complete NNS Governance refreshes.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsPagedRefreshOptions {
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) page_size: u32,
    pub(in crate::nns) max_pages: Option<u32>,
}

impl NnsPagedRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            common,
            page_size: required_typed(matches, "page-size"),
            max_pages: typed_option(matches, "max-pages"),
        }
    }

    pub(in crate::nns) fn into_request(
        self,
        cache_root: PathBuf,
        now_unix_secs: u64,
    ) -> NnsGovernanceRefreshRequest {
        NnsGovernanceRefreshRequest::new(
            cache_root,
            self.common.network,
            self.common.source_endpoint,
            now_unix_secs,
            self.page_size,
        )
        .with_max_pages(self.max_pages)
    }
}
