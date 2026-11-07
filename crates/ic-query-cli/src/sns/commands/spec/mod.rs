//! Clap command definitions for explicitly targeted SNS neuron reads and caches.

use crate::cli::{
    clap::with_options,
    common::{
        COLLECTION_MODE_CACHE_ONLY, COLLECTION_MODE_FORCE_REFRESH, COLLECTION_MODE_LIVE,
        COLLECTION_MODE_LIVE_OR_CACHE_BY_VIEW, collection_help, json_arg,
    },
};
use clap::Command as ClapCommand;
use ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT;

const SNS_NEURONS_HELP_AFTER: &str = "\
Examples:
  icq sns neuron list <governance-canister-id>
  icq sns neuron list <governance-canister-id> --limit 10
  icq sns neuron list <governance-canister-id> --owner zqfso-syaaa-aaaaq-aaafq-cai
  icq sns neuron list <governance-canister-id> --verbose
  icq sns neuron refresh <governance-canister-id>
  icq sns neuron cache list
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron list <governance-canister-id> --limit 500 --sort stake
  icq --network ic sns neuron list <governance-canister-id> --json";

const SNS_NEURON_INFO_HELP_AFTER: &str = "\
Examples:
  icq sns neuron info <governance-canister-id> 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
  icq --network ic sns neuron info <governance-canister-id> 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f --json";

const SNS_NEURONS_CACHE_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache list
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron cache status <governance-canister-id> --json";

const SNS_NEURONS_CACHE_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache list
  icq sns neuron cache list --json";

const SNS_NEURONS_CACHE_STATUS_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron cache status <governance-canister-id> --json";

const SNS_NEURONS_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq sns neuron refresh <governance-canister-id>
  icq sns neuron refresh <governance-canister-id>
  icq sns neuron refresh <governance-canister-id> --page-size 100
  icq --network ic sns neuron refresh <governance-canister-id> --json";

pub(super) fn sns_command() -> ClapCommand {
    ClapCommand::new("sns")
        .about("Inspect neuron ownership and stake")
        .subcommand(sns_neuron_command())
}

fn sns_neuron_command() -> ClapCommand {
    ClapCommand::new("neuron")
        .about("List, inspect, and refresh SNS governance neurons")
        .subcommand(sns_neuron_list_command())
        .subcommand(sns_neuron_info_command())
        .subcommand(sns_neuron_refresh_command())
        .subcommand(sns_neuron_cache_command())
}

fn sns_neuron_info_command() -> ClapCommand {
    with_options::<crate::sns::commands::options::SnsNeuronOptions>(
        ClapCommand::new("info").about("Show one neuron stake and permissions by exact 32-byte id"),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_SNS_SOURCE_ENDPOINT)
            .help("IC API endpoint used for Governance queries")
    })
    .after_help(collection_help(
        COLLECTION_MODE_LIVE,
        SNS_NEURON_INFO_HELP_AFTER,
    ))
}

fn sns_neuron_list_command() -> ClapCommand {
    with_options::<crate::sns::commands::options::SnsListOptions>(
        ClapCommand::new("list")
            .about("List SNS governance neurons by Governance canister principal"),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_SNS_SOURCE_ENDPOINT)
            .help("IC API endpoint used for Governance neuron queries")
    })
    .after_help(collection_help(
        COLLECTION_MODE_LIVE_OR_CACHE_BY_VIEW,
        SNS_NEURONS_HELP_AFTER,
    ))
}

fn sns_neuron_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .about("Inspect local complete SNS governance neuron snapshots")
        .subcommand(sns_neuron_cache_list_command())
        .subcommand(sns_neuron_cache_status_command())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_NEURONS_CACHE_HELP_AFTER,
        ))
}

fn sns_neuron_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .about("List local complete SNS neuron snapshots")
        .arg(json_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_NEURONS_CACHE_LIST_HELP_AFTER,
        ))
}

fn sns_neuron_cache_status_command() -> ClapCommand {
    with_options::<crate::sns::commands::options::SnsCacheStatusOptions>(
        ClapCommand::new("status")
            .about("Show local SNS neuron snapshot and refresh-attempt status"),
    )
    .after_help(collection_help(
        COLLECTION_MODE_CACHE_ONLY,
        SNS_NEURONS_CACHE_STATUS_HELP_AFTER,
    ))
}

fn sns_neuron_refresh_command() -> ClapCommand {
    with_options::<crate::sns::commands::options::SnsRefreshOptions>(
        ClapCommand::new("refresh")
            .about("Force-refresh and cache a complete SNS governance neuron snapshot"),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_SNS_SOURCE_ENDPOINT)
            .help("IC API endpoint used for Governance neuron queries")
    })
    .after_help(collection_help(
        COLLECTION_MODE_FORCE_REFRESH,
        SNS_NEURONS_REFRESH_HELP_AFTER,
    ))
}
