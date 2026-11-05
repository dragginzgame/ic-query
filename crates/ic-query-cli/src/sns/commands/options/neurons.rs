//! Parse SNS neuron commands directly into acquisition-specific report requests.

use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option},
        common::OutputFormat,
    },
    sns::commands::{SnsCommandError, options::lookup_request},
};
use clap::ArgMatches;
use ic_platform::candid::contract::Principal;
use ic_query::sns::SnsNeuronRequest;
use ic_query::sns::SnsNeuronsCacheSort;
use ic_query::sns::SnsNeuronsRequest;
use std::path::PathBuf;

const SNS_NEURONS_LIVE_MAX_LIMIT: u32 = 100;

pub(in crate::sns::commands) fn neuron_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> Result<(OutputFormat, SnsNeuronRequest), crate::sns::commands::SnsCommandError> {
    let (format, lookup) = lookup_request(matches, network, now_unix_secs)?;
    Ok((
        format,
        SnsNeuronRequest::new(lookup, required_string(matches, "neuron-id")),
    ))
}

pub(in crate::sns::commands) fn neurons_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
    cache_root: impl FnOnce() -> Result<PathBuf, SnsCommandError>,
) -> Result<(OutputFormat, SnsNeuronsRequest), SnsCommandError> {
    let (format, lookup) = lookup_request(matches, network, now_unix_secs)?;
    let limit = required_typed(matches, "limit");
    let owner_principal_id =
        typed_option::<Principal>(matches, "owner").map(|principal| principal.to_text());
    let cache_sort = required_typed::<Option<SnsNeuronsCacheSort>>(matches, "sort");
    // Clap relationships concern argument presence, not another argument's value
    // or a range conditional on it. Keep these route-specific checks together.
    let request = if let Some(sort) = cache_sort {
        if owner_principal_id.is_some() {
            return Err(SnsCommandError::Usage("`--owner` is supported only with `icq sns neuron list --sort api`; cached `--sort <id|stake|maturity|created>` views read the complete full-neuron cache and do not accept owner filtering".to_string()));
        }
        SnsNeuronsRequest::cached(lookup, limit, cache_root()?, sort)
    } else {
        if limit > SNS_NEURONS_LIVE_MAX_LIMIT {
            return Err(SnsCommandError::Usage(format!(
                "`icq sns neuron list --sort api` can request at most {SNS_NEURONS_LIVE_MAX_LIMIT} live neurons at a time; refresh the cache and use `--sort <id|stake|maturity|created>` for larger limits"
            )));
        }
        SnsNeuronsRequest::live(lookup, limit, owner_principal_id)
    };
    Ok((format, request))
}
