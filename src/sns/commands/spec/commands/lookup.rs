use super::args::sns_lookup_input_arg;
use crate::{
    cli::{
        common::{format_arg, source_endpoint_arg},
        globals::internal_network_arg,
    },
    sns::report::DEFAULT_SNS_SOURCE_ENDPOINT,
};
use clap::Command as ClapCommand;

const SNS_INFO_HELP_AFTER: &str = "\
Examples:
  icq sns info 1
  icq sns info 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns info 1 --format json";

const SNS_TOKEN_HELP_AFTER: &str = "\
Examples:
  icq sns token 1
  icq sns token 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns token 1 --format json";

const SNS_PARAMS_HELP_AFTER: &str = "\
Examples:
  icq sns params 1
  icq sns params 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns params 1 --format json";

pub(in crate::sns::commands) fn sns_info_command() -> ClapCommand {
    sns_lookup_command(
        "info",
        "icq sns info",
        "Resolve a deployed SNS by list id or root principal",
        "IC API endpoint used for SNS-W and governance metadata queries",
        SNS_INFO_HELP_AFTER,
    )
}

pub(in crate::sns::commands) fn sns_token_command() -> ClapCommand {
    sns_lookup_command(
        "token",
        "icq sns token",
        "Show SNS ledger token metadata by list id or root principal",
        "IC API endpoint used for SNS-W, governance, and ledger queries",
        SNS_TOKEN_HELP_AFTER,
    )
}

pub(in crate::sns::commands) fn sns_params_command() -> ClapCommand {
    sns_lookup_command(
        "params",
        "icq sns params",
        "Show SNS governance nervous system parameters by list id or root principal",
        "IC API endpoint used for SNS-W and governance queries",
        SNS_PARAMS_HELP_AFTER,
    )
}

fn sns_lookup_command(
    name: &'static str,
    bin_name: &'static str,
    about: &'static str,
    source_endpoint_help: &'static str,
    after_help: &'static str,
) -> ClapCommand {
    ClapCommand::new(name)
        .bin_name(bin_name)
        .about(about)
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT).help(source_endpoint_help))
        .arg(internal_network_arg().default_value("ic"))
        .after_help(after_help)
}
