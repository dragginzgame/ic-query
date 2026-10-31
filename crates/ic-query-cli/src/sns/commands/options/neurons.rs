//! Parse SNS neuron commands directly into acquisition-specific report requests.

use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option},
        common::OutputFormat,
    },
    sns::commands::{SnsCommandError, options::SnsLookupOptions, spec::SnsNeuronsSortArg},
};
use clap::ArgMatches;
use ic_platform::candid::contract::Principal;
use ic_query::sns::{SnsNeuronRequest, SnsNeuronsCacheSort, SnsNeuronsRequest};
use std::path::PathBuf;

const SNS_NEURONS_LIVE_MAX_LIMIT: u32 = 100;

pub(in crate::sns::commands) fn neuron_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> (OutputFormat, SnsNeuronRequest) {
    let (format, lookup) =
        SnsLookupOptions::from_matches(matches, network).into_request(now_unix_secs);
    (
        format,
        SnsNeuronRequest::new(lookup, required_string(matches, "neuron-id")),
    )
}

pub(in crate::sns::commands) fn neurons_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
    cache_root: impl FnOnce() -> Result<PathBuf, SnsCommandError>,
) -> Result<(OutputFormat, SnsNeuronsRequest), SnsCommandError> {
    let (format, lookup) =
        SnsLookupOptions::from_matches(matches, network).into_request(now_unix_secs);
    let limit = required_typed(matches, "limit");
    let owner_principal_id =
        typed_option::<Principal>(matches, "owner").map(|principal| principal.to_text());
    let cache_sort = match required_typed(matches, "sort") {
        SnsNeuronsSortArg::Api => None,
        SnsNeuronsSortArg::Id => Some(SnsNeuronsCacheSort::Id),
        SnsNeuronsSortArg::Stake => Some(SnsNeuronsCacheSort::Stake),
        SnsNeuronsSortArg::Maturity => Some(SnsNeuronsCacheSort::Maturity),
        SnsNeuronsSortArg::Created => Some(SnsNeuronsCacheSort::Created),
    };
    let request = if let Some(sort) = cache_sort {
        if owner_principal_id.is_some() {
            return Err(SnsCommandError::Usage("`--owner` is supported only with `icq sns neuron list --sort api`; cached `--sort <id|stake|maturity|created>` views read the complete full-neuron cache and do not accept owner filtering".to_string()));
        }
        Principal::from_text(&lookup.input).map_err(|_| {
            SnsCommandError::Usage(
                "cached neuron views require an SNS root canister principal".to_string(),
            )
        })?;
        SnsNeuronsRequest::cached(lookup, limit, cache_root()?, sort)
    } else {
        if limit > SNS_NEURONS_LIVE_MAX_LIMIT {
            return Err(SnsCommandError::Usage(format!(
                "`icq sns neuron list --sort api` can request at most {SNS_NEURONS_LIVE_MAX_LIMIT} live neurons at a time; refresh the cache and use `--sort <id|stake|maturity|created>` for larger limits"
            )));
        }
        SnsNeuronsRequest::live(lookup, limit, owner_principal_id)
    };
    Ok((format, request.with_verbose(matches.get_flag("verbose"))))
}
