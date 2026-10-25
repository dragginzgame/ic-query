//! Module: sns::commands::options::lookup
//!
//! Responsibility: parse shared SNS selector, source, and collection options.
//! Does not own: command-specific option fields or report requests.
//! Boundary: captures common lookup, cache selection, and pagination inputs.

use crate::cli::{
    clap::{required_string, required_typed, typed_option},
    common::{NetworkOutputOptions, OutputFormat, output_format},
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsCacheListRequest, SnsCacheStatusRequest, SnsLookupRequest, SnsSnapshotRefreshRequest,
};
use std::path::PathBuf;

///
/// SnsLookupOptions
///
/// Common selector and source options shared by SNS lookup commands.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsLookupOptions {
    pub(in crate::sns::commands) input: String,
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
    pub(in crate::sns::commands) source_endpoint: String,
}

impl SnsLookupOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: network.to_string(),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

///
/// SnsPagedRefreshOptions
///
/// Common SNS selector and page bounds for complete Governance refreshes.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsPagedRefreshOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) page_size: u32,
    pub(in crate::sns::commands) max_pages: Option<u32>,
}

impl SnsPagedRefreshOptions {
    pub(in crate::sns::commands) fn into_request(
        self,
        cache_root: PathBuf,
        now_unix_secs: u64,
    ) -> (OutputFormat, SnsSnapshotRefreshRequest) {
        (
            self.lookup.format,
            SnsSnapshotRefreshRequest {
                lookup: SnsLookupRequest::new(
                    self.lookup.network,
                    self.lookup.source_endpoint,
                    now_unix_secs,
                    self.lookup.input,
                ),
                cache_root,
                page_size: self.page_size,
                max_pages: self.max_pages,
            },
        )
    }

    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            page_size: required_typed(matches, "page-size"),
            max_pages: typed_option::<u32>(matches, "max-pages"),
        }
    }
}

///
/// SnsCacheStatusOptions
///
/// Common selector and output options for local SNS Governance cache status.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsCacheStatusOptions {
    pub(in crate::sns::commands) input: String,
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
}

impl SnsCacheStatusOptions {
    pub(in crate::sns::commands) fn into_request(
        self,
        cache_root: PathBuf,
    ) -> (OutputFormat, SnsCacheStatusRequest) {
        (
            self.format,
            SnsCacheStatusRequest {
                network: self.network,
                cache_root,
                input: self.input,
            },
        )
    }

    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: network.to_string(),
            format: output_format(matches),
        }
    }
}

pub(in crate::sns::commands) fn cache_list_request(
    options: NetworkOutputOptions,
    cache_root: PathBuf,
) -> (OutputFormat, SnsCacheListRequest) {
    (
        options.format,
        SnsCacheListRequest {
            network: options.network,
            cache_root,
        },
    )
}
