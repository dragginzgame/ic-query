//! Module: sns::commands::spec::commands::neurons::root
//!
//! Responsibility: build the clap specs for `icq sns neuron` and its list view.
//! Does not own: neuron list execution, cache selection, or report output.
//! Boundary: defines list options, owner filtering input, and examples.

use crate::{
    cli::{
        clap::{flag_arg, value_arg},
        common::{
            COLLECTION_MODE_LIVE, COLLECTION_MODE_LIVE_OR_CACHE_BY_VIEW, collection_help, json_arg,
            source_endpoint_arg,
        },
    },
    sns::commands::spec::commands::{
        args::{governance_input_arg, sns_neuron_id_value_parser},
        neurons::sort::neurons_sort_arg,
    },
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT;

const SNS_NEURONS_DEFAULT_LIMIT: &str = "25";

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

pub(in crate::sns::commands) fn sns_neuron_command() -> ClapCommand {
    ClapCommand::new("neuron")
        .about("List, inspect, and refresh SNS governance neurons")
        .subcommand(sns_neuron_list_command())
        .subcommand(sns_neuron_info_command())
        .subcommand(super::sns_neuron_refresh_command())
        .subcommand(super::sns_neuron_cache_command())
}

pub(in crate::sns::commands) fn sns_neuron_info_command() -> ClapCommand {
    ClapCommand::new("info")
        .about("Show one neuron stake and permissions by exact 32-byte id")
        .arg(governance_input_arg())
        .arg(
            value_arg("neuron-id")
                .value_name("neuron-id")
                .required(true)
                .value_parser(sns_neuron_id_value_parser())
                .help("Exact 32-byte SNS neuron id as lowercase hexadecimal"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for Governance queries"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            SNS_NEURON_INFO_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_neuron_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .about("List SNS governance neurons by Governance canister principal")
        .arg(governance_input_arg())
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for Governance neuron queries"),
        )
        .arg(
            value_arg("limit")
                .long("limit")
                .value_name("count")
                .default_value(SNS_NEURONS_DEFAULT_LIMIT)
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Maximum rows to show; --sort api can request at most 100 live neurons"),
        )
        .arg(
            value_arg("owner")
                .long("owner")
                .value_name("principal")
                .value_parser(clap::value_parser!(
                    ic_platform::candid::contract::Principal
                ))
                .help("Filter neurons having permissions for this principal"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full neuron IDs in text output"),
        )
        .arg(neurons_sort_arg())
        .after_help(collection_help(
            COLLECTION_MODE_LIVE_OR_CACHE_BY_VIEW,
            SNS_NEURONS_HELP_AFTER,
        ))
}
