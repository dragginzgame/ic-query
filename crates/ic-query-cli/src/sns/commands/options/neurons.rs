//! Parse SNS neuron commands directly into acquisition-specific report requests.

use crate::cli::clap::parsed_options;
use crate::{cli::common::OutputFormat, sns::commands::SnsCommandError};
use clap::ArgMatches;
use ic_platform::candid::contract::Principal;
use ic_query::sns::SnsNeuronRequest;
use ic_query::sns::SnsNeuronsCacheSort;
use ic_query::sns::SnsNeuronsRequest;
use std::path::PathBuf;

const SNS_NEURONS_LIVE_MAX_LIMIT: u32 = 100;

///
/// SnsNeuronOptions
///
/// Exact neuron identity and its explicit Governance source.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::sns::commands) struct SnsNeuronOptions {
    #[command(flatten)]
    pub(in crate::sns::commands) target: super::lookup::GovernanceInput,
    #[arg(id = "neuron-id", value_name = "neuron-id", value_parser = sns_neuron_id_value_parser(), help = "Exact 32-byte SNS neuron id as lowercase hexadecimal")]
    pub(in crate::sns::commands) neuron_id: String,
    #[command(flatten)]
    pub(in crate::sns::commands) source: crate::cli::common::SourceOutputOptions,
}

///
/// SnsListOptions
///
/// Typed live-or-cache view controls over the same Governance target.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::sns::commands) struct SnsListOptions {
    #[command(flatten)]
    pub(in crate::sns::commands) lookup: super::lookup::SnsLookupOptions,
    #[arg(long, value_name = "count", default_value = "25", value_parser = clap::builder::RangedU64ValueParser::<u32>::new().range(1..), help = "Maximum rows to show; --sort api can request at most 100 live neurons")]
    pub(in crate::sns::commands) limit: u32,
    #[arg(
        long,
        value_name = "principal",
        help = "Filter neurons having permissions for this principal"
    )]
    pub(in crate::sns::commands) owner: Option<Principal>,
    #[arg(long, help = "Show full neuron IDs in text output")]
    pub(in crate::sns::commands) verbose: bool,
    // None is the explicit `api` value, not an omitted optional argument.
    #[arg(long, value_name = "api|id|stake|maturity|created", default_value = "api", value_parser = crate::cli::clap::choice_parser(&[None, Some(SnsNeuronsCacheSort::Id), Some(SnsNeuronsCacheSort::Stake), Some(SnsNeuronsCacheSort::Maturity), Some(SnsNeuronsCacheSort::Created)], |sort| sort.map_or("api", SnsNeuronsCacheSort::as_str)), help = "Row order; api uses a bounded live query, other sorts read the complete cache")]
    pub(in crate::sns::commands) sort: std::option::Option<SnsNeuronsCacheSort>,
}

fn sns_neuron_id_value_parser() -> clap::builder::ValueParser {
    clap::builder::ValueParser::new(|value: &str| {
        let canonical = value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
        canonical
            .then(|| value.to_string())
            .ok_or_else(|| "must be exactly 64 lowercase hexadecimal characters".to_string())
    })
}

pub(in crate::sns::commands) fn neuron_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
) -> Result<(OutputFormat, SnsNeuronRequest), crate::sns::commands::SnsCommandError> {
    let options: SnsNeuronOptions = parsed_options(matches);
    let (format, lookup) = super::lookup::SnsLookupOptions {
        target: options.target,
        source: options.source,
    }
    .into_target(network, now_unix_secs)?;
    Ok((format, SnsNeuronRequest::new(lookup, options.neuron_id)))
}

pub(in crate::sns::commands) fn neurons_request(
    matches: &ArgMatches,
    network: &str,
    now_unix_secs: u64,
    cache_root: impl FnOnce() -> Result<PathBuf, SnsCommandError>,
) -> Result<(OutputFormat, SnsNeuronsRequest), SnsCommandError> {
    let options: SnsListOptions = parsed_options(matches);
    let (format, lookup) = options.lookup.into_target(network, now_unix_secs)?;
    let limit = options.limit;
    let owner_principal_id = options.owner.map(|principal| principal.to_text());
    let cache_sort = options.sort;
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
