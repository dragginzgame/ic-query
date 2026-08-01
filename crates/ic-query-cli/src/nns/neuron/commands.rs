//! Clap specifications for public NNS neuron commands.

use crate::{
    cli::{
        clap::{flag_arg, passthrough_subcommand, render_help, value_arg},
        common::{
            COLLECTION_MODE_CACHE_ONLY, COLLECTION_MODE_CACHE_PREFERRED_LIVE_FALLBACK,
            COLLECTION_MODE_FORCE_REFRESH, collection_help,
        },
    },
    nns::leaf,
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::nns::neuron::{DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, NNS_NEURON_MAX_PAGE_SIZE};

const LIST_DEFAULT_LIMIT: &str = "25";
const REFRESH_DEFAULT_PAGE_SIZE: &str = "300";

const NEURON_HELP_AFTER: &str = "\
Examples:
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

pub(super) fn neuron_command() -> ClapCommand {
    ClapCommand::new("neuron")
        .bin_name("icq nns neuron")
        .about("Inspect public NNS Governance neuron views")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List public NNS neurons"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("info").about("Show one public NNS neuron"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("refresh").about("Refresh the complete public NNS neuron snapshot"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("cache").about("Inspect the local public NNS neuron snapshot"),
        ))
        .after_help(NEURON_HELP_AFTER)
}

pub(super) fn neuron_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq nns neuron list")
        .about("List public NNS neurons")
        .disable_help_flag(true)
        .arg(leaf::json_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_NEURON_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native NNS Governance query"),
        )
        .arg(
            value_arg("limit")
                .long("limit")
                .value_name("count")
                .default_value(LIST_DEFAULT_LIMIT)
                .value_parser(
                    RangedU64ValueParser::<u32>::new()
                        .range(1..=u64::from(NNS_NEURON_MAX_PAGE_SIZE)),
                )
                .help("Maximum public neurons to return"),
        )
        .arg(
            value_arg("start-neuron-id")
                .long("start-neuron-id")
                .value_name("neuron-id")
                .value_parser(RangedU64ValueParser::<u64>::new().range(1..))
                .help("Return neurons with ids greater than this exclusive cursor"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show expanded public neuron details in text output"),
        )
        .arg(leaf::network_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_PREFERRED_LIVE_FALLBACK,
            LIST_HELP_AFTER,
        ))
}

pub(super) fn neuron_info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq nns neuron info")
        .about("Show one public NNS neuron")
        .disable_help_flag(true)
        .arg(
            value_arg("neuron-id")
                .value_name("neuron-id")
                .required(true)
                .value_parser(RangedU64ValueParser::<u64>::new().range(1..))
                .help("NNS Governance neuron id"),
        )
        .arg(leaf::json_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_NEURON_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native NNS Governance query"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show expanded public neuron details in text output"),
        )
        .arg(leaf::network_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_PREFERRED_LIVE_FALLBACK,
            INFO_HELP_AFTER,
        ))
}

pub(super) fn neuron_refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq nns neuron refresh")
        .about("Refresh the complete public NNS neuron snapshot")
        .disable_help_flag(true)
        .arg(leaf::json_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_NEURON_SOURCE_ENDPOINT)
                .help("IC API endpoint used for every native NNS Governance page"),
        )
        .arg(
            value_arg("page-size")
                .long("page-size")
                .value_name("count")
                .default_value(REFRESH_DEFAULT_PAGE_SIZE)
                .value_parser(
                    RangedU64ValueParser::<u32>::new()
                        .range(1..=u64::from(NNS_NEURON_MAX_PAGE_SIZE)),
                )
                .help("Maximum public neurons requested per Governance page"),
        )
        .arg(
            value_arg("max-pages")
                .long("max-pages")
                .value_name("count")
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Stop without publishing if this cap precedes API exhaustion"),
        )
        .arg(leaf::network_arg())
        .after_help(collection_help(
            COLLECTION_MODE_FORCE_REFRESH,
            REFRESH_HELP_AFTER,
        ))
}

pub(super) fn neuron_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq nns neuron cache")
        .about("Inspect the local complete public NNS neuron snapshot")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("status").about("Show snapshot and refresh-attempt status"),
        ))
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            CACHE_HELP_AFTER,
        ))
}

pub(super) fn neuron_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .bin_name("icq nns neuron cache status")
        .about("Show public NNS neuron snapshot and refresh-attempt status")
        .disable_help_flag(true)
        .arg(leaf::json_arg())
        .arg(leaf::network_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            CACHE_HELP_AFTER,
        ))
}

pub(super) fn neuron_usage_for_error() -> String {
    render_help(neuron_command())
}

pub(super) fn neuron_list_usage_for_error() -> String {
    render_help(neuron_list_command())
}

pub(super) fn neuron_info_usage_for_error() -> String {
    render_help(neuron_info_command())
}

pub(super) fn neuron_refresh_usage_for_error() -> String {
    render_help(neuron_refresh_command())
}

pub(super) fn neuron_cache_usage_for_error() -> String {
    render_help(neuron_cache_command())
}

pub(super) fn neuron_cache_status_usage_for_error() -> String {
    render_help(neuron_cache_status_command())
}

#[cfg(test)]
pub(in crate::nns) fn neuron_usage() -> String {
    neuron_usage_for_error()
}

#[cfg(test)]
pub(in crate::nns) fn neuron_list_usage() -> String {
    neuron_list_usage_for_error()
}

#[cfg(test)]
pub(in crate::nns) fn neuron_info_usage() -> String {
    neuron_info_usage_for_error()
}

#[cfg(test)]
pub(in crate::nns) fn neuron_refresh_usage() -> String {
    neuron_refresh_usage_for_error()
}

#[cfg(test)]
pub(in crate::nns) fn neuron_cache_usage() -> String {
    neuron_cache_usage_for_error()
}

#[cfg(test)]
pub(in crate::nns) fn neuron_cache_status_usage() -> String {
    neuron_cache_status_usage_for_error()
}
