//! Clap specifications for public NNS neuron commands.

use crate::cli::clap::with_options;
use crate::{
    cli::common::{
        COLLECTION_MODE_CACHE_ONLY, COLLECTION_MODE_CACHE_PREFERRED_LIVE_FALLBACK,
        COLLECTION_MODE_FORCE_REFRESH, collection_help,
    },
    nns::leaf,
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::nns::neuron::DEFAULT_NNS_NEURON_SOURCE_ENDPOINT;
use ic_query::nns::neuron::NNS_NEURON_MAX_PAGE_SIZE;

const LIST_DEFAULT_LIMIT: &str = "25";
const REFRESH_DEFAULT_PAGE_SIZE: &str = "300";

const NEURON_HELP_AFTER: &str = "\
Examples:
  icq nns neuron distribution
  icq nns neuron distribution --json
  icq nns neuron list
  icq nns neuron info 123456789
  icq nns neuron refresh
  icq nns neuron cache status";

const LIST_HELP_AFTER: &str = "\
Examples:
  icq nns neuron list
  icq nns neuron list --limit 100 --start-neuron-id 123456789
  icq nns neuron list --verbose
  icq nns neuron list --json";

const INFO_HELP_AFTER: &str = "\
Examples:
  icq nns neuron info 123456789
  icq nns neuron info 123456789 --verbose
  icq nns neuron info 123456789 --json";

const REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns neuron refresh
  icq nns neuron refresh --page-size 300
  icq nns neuron refresh --max-pages 5
  icq nns neuron refresh --json";

const CACHE_HELP_AFTER: &str = "\
Examples:
  icq nns neuron cache status
  icq nns neuron cache status --json";

pub(in crate::nns) fn neuron_command() -> ClapCommand {
    ClapCommand::new("neuron")
        .about("Inspect public NNS Governance neuron views")
        .subcommand(neuron_list_command())
        .subcommand(neuron_distribution_command())
        .subcommand(neuron_info_command())
        .subcommand(neuron_refresh_command())
        .subcommand(neuron_cache_command())
        .after_help(NEURON_HELP_AFTER)
}

pub(in crate::nns) fn neuron_distribution_command() -> ClapCommand {
    ClapCommand::new("distribution")
        .about("Summarize public-neuron distribution from the complete local snapshot")
        .arg(leaf::json_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            "Examples:\n  icq nns neuron distribution\n  icq nns neuron distribution --json",
        ))
}

///
/// NeuronListOptions
///
/// Typed arguments for the neuron_list_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct NeuronListOptions {
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
    #[arg(long, value_name = "count", default_value = LIST_DEFAULT_LIMIT, value_parser = RangedU64ValueParser::<u32>::new() .range(1..=u64::from(NNS_NEURON_MAX_PAGE_SIZE)), help = "Maximum public neurons to return")]
    pub(super) limit: u32,
    #[arg(id = "start-neuron-id", long, value_name = "neuron-id", value_parser = RangedU64ValueParser::<u64>::new().range(1..), help = "Return neurons with ids greater than this exclusive cursor")]
    pub(super) start_neuron_id: Option<u64>,
    #[arg(long, help = "Show expanded public neuron details in text output")]
    pub(super) verbose: bool,
}

pub(in crate::nns) fn neuron_list_command() -> ClapCommand {
    with_options::<NeuronListOptions>(
        ClapCommand::new("list")
            .about("List public NNS neurons")
            .after_help(collection_help(
                COLLECTION_MODE_CACHE_PREFERRED_LIVE_FALLBACK,
                LIST_HELP_AFTER,
            )),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_NNS_NEURON_SOURCE_ENDPOINT)
            .help("IC API endpoint used for the native NNS Governance query")
    })
}

///
/// NeuronInfoOptions
///
/// Typed arguments for the neuron_info_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct NeuronInfoOptions {
    #[arg(id = "neuron-id", value_name = "neuron-id", value_parser = RangedU64ValueParser::<u64>::new().range(1..), help = "NNS Governance neuron id")]
    pub(super) neuron_id: u64,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
    #[arg(long, help = "Show expanded public neuron details in text output")]
    pub(super) verbose: bool,
}

pub(in crate::nns) fn neuron_info_command() -> ClapCommand {
    with_options::<NeuronInfoOptions>(
        ClapCommand::new("info")
            .about("Show one public NNS neuron")
            .after_help(collection_help(
                COLLECTION_MODE_CACHE_PREFERRED_LIVE_FALLBACK,
                INFO_HELP_AFTER,
            )),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_NNS_NEURON_SOURCE_ENDPOINT)
            .help("IC API endpoint used for the native NNS Governance query")
    })
}

pub(in crate::nns) fn neuron_refresh_command() -> ClapCommand {
    with_options::<crate::nns::leaf::NnsPagedRefreshOptions>(
        ClapCommand::new("refresh")
            .about("Refresh the complete public NNS neuron snapshot")
            .after_help(collection_help(
                COLLECTION_MODE_FORCE_REFRESH,
                REFRESH_HELP_AFTER,
            )),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_NNS_NEURON_SOURCE_ENDPOINT)
            .help("IC API endpoint used for every native NNS Governance page")
    })
    .mut_arg("page-size", |arg| {
        arg.default_value(REFRESH_DEFAULT_PAGE_SIZE)
            .value_parser(
                clap::builder::RangedU64ValueParser::<u32>::new()
                    .range(1..=u64::from(NNS_NEURON_MAX_PAGE_SIZE)),
            )
            .help("Maximum public neurons requested per Governance page")
    })
    .mut_arg("max-pages", |arg| {
        arg.value_parser(clap::builder::RangedU64ValueParser::<u32>::new().range(1..))
            .help("Stop without publishing if this cap precedes API exhaustion")
    })
}

pub(in crate::nns) fn neuron_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .about("Inspect the local complete public NNS neuron snapshot")
        .subcommand(neuron_cache_status_command())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            CACHE_HELP_AFTER,
        ))
}

pub(in crate::nns) fn neuron_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .about("Show public NNS neuron snapshot and refresh-attempt status")
        .arg(leaf::json_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            CACHE_HELP_AFTER,
        ))
}
