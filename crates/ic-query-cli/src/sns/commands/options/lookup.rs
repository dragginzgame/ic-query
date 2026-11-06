//! Module: sns::commands::options::lookup
//!
//! Responsibility: parse shared SNS selector, source, and collection options.
//! Does not own: command-specific fields, clock reads, or cache-root selection.
//! Boundary: constructs shared lookup, cache selection, and pagination requests.

use crate::cli::clap::parsed_options;
use crate::cli::common::{NetworkOutputOptions, OutputFormat};
use clap::ArgMatches;
use ic_query::sns::SnsCacheListRequest;
use ic_query::sns::SnsCacheStatusRequest;
use ic_query::sns::SnsNeuronTargetRequest;
use ic_query::sns::SnsSnapshotRefreshRequest;
use std::path::PathBuf;

///
/// GovernanceInput
///
/// Explicit canonical Governance principal shared by live and cache operations.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::sns::commands) struct GovernanceInput {
    #[arg(value_name = "governance-canister-id", value_parser = crate::cli::clap::principal_text_value_parser(), help = "Governance canister principal holding the neurons")]
    pub(in crate::sns::commands) input: String,
}

///
/// SnsLookupOptions
///
/// Governance target and source/output arguments for native SNS reads.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::sns::commands) struct SnsLookupOptions {
    #[command(flatten)]
    pub(in crate::sns::commands) target: GovernanceInput,
    #[command(flatten)]
    pub(in crate::sns::commands) source: crate::cli::common::SourceOutputOptions,
}

impl SnsLookupOptions {
    pub(in crate::sns::commands) fn into_target(
        self,
        network: &str,
        now: u64,
    ) -> Result<(OutputFormat, SnsNeuronTargetRequest), crate::sns::commands::SnsCommandError> {
        Ok((
            self.source.format(),
            SnsNeuronTargetRequest::new(
                network,
                self.source.source_endpoint,
                now,
                self.target.input,
            )?,
        ))
    }
}

///
/// SnsRefreshOptions
///
/// Bounded, explicit complete-neuron refresh arguments.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::sns::commands) struct SnsRefreshOptions {
    #[command(flatten)]
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    #[arg(id = "page-size", long, value_name = "count", default_value = "100", value_parser = clap::builder::RangedU64ValueParser::<u32>::new().range(1..=u64::from(ic_query::sns::SNS_REFRESH_MAX_PAGE_SIZE)), help = "Maximum neurons to request per SNS governance page")]
    pub(in crate::sns::commands) page_size: u32,
    #[arg(id = "max-pages", long, value_name = "count", value_parser = clap::builder::RangedU64ValueParser::<u32>::new().range(1..), help = "Stop before publishing if this page count is reached before API exhaustion")]
    pub(in crate::sns::commands) max_pages: Option<u32>,
}

///
/// SnsCacheStatusOptions
///
/// Local cache selection without a network endpoint or live call.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::sns::commands) struct SnsCacheStatusOptions {
    #[command(flatten)]
    pub(in crate::sns::commands) target: GovernanceInput,
    #[arg(long, help = "Print raw JSON instead of human-readable text")]
    pub(in crate::sns::commands) json: bool,
}

pub(in crate::sns::commands) fn snapshot_refresh_request(
    matches: &ArgMatches,
    network: &str,
    cache_root: PathBuf,
    now_unix_secs: u64,
) -> Result<(OutputFormat, SnsSnapshotRefreshRequest), crate::sns::commands::SnsCommandError> {
    let options: SnsRefreshOptions = parsed_options(matches);
    let (format, target) = options.lookup.into_target(network, now_unix_secs)?;
    Ok((
        format,
        SnsSnapshotRefreshRequest {
            target,
            cache_root,
            page_size: options.page_size,
            max_pages: options.max_pages,
        },
    ))
}

pub(in crate::sns::commands) fn cache_status_request(
    matches: &ArgMatches,
    network: &str,
    cache_root: PathBuf,
) -> (OutputFormat, SnsCacheStatusRequest) {
    let options: SnsCacheStatusOptions = parsed_options(matches);
    (
        OutputFormat::from_json(options.json),
        SnsCacheStatusRequest {
            network: network.to_string(),
            cache_root,
            input: options.target.input,
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
