//! Module: sns::commands::options::lookup
//!
//! Responsibility: parse shared SNS selector, source, and collection options.
//! Does not own: command-specific fields, clock reads, or cache-root selection.
//! Boundary: constructs shared lookup, cache selection, and pagination requests.

use crate::cli::{
    clap::{required_string, required_typed, typed_option},
    common::{NetworkOutputOptions, OutputFormat, output_format},
};
use clap::ArgMatches;
use ic_query::sns::SnsCacheListRequest;
use ic_query::sns::SnsCacheStatusRequest;
use ic_query::sns::SnsLookupRequest;
use ic_query::sns::SnsSnapshotRefreshRequest;
use std::path::PathBuf;

pub(in crate::sns::commands) fn lookup_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, SnsLookupRequest) {
    (
        output_format(matches),
        SnsLookupRequest::new(
            network,
            required_string(matches, "source-endpoint"),
            now_unix_secs,
            required_string(matches, "input"),
        ),
    )
}

pub(in crate::sns::commands) fn snapshot_refresh_request(
    matches: &ArgMatches,
    network: &str,
    cache_root: PathBuf,
    now_unix_secs: u64,
) -> (OutputFormat, SnsSnapshotRefreshRequest) {
    let (format, lookup) = lookup_request(matches, network, now_unix_secs);
    (
        format,
        SnsSnapshotRefreshRequest {
            lookup,
            cache_root,
            page_size: required_typed(matches, "page-size"),
            max_pages: typed_option(matches, "max-pages"),
        },
    )
}

pub(in crate::sns::commands) fn cache_status_request(
    matches: &ArgMatches,
    network: &str,
    cache_root: PathBuf,
) -> (OutputFormat, SnsCacheStatusRequest) {
    (
        output_format(matches),
        SnsCacheStatusRequest {
            network: network.to_string(),
            cache_root,
            input: required_string(matches, "input"),
        },
    )
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
