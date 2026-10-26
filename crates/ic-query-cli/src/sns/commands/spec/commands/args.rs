//! Module: sns::commands::spec::commands::args
//!
//! Responsibility: provide shared SNS clap argument builders and value parsers.
//! Does not own: command specs, option DTOs, or command runtime validation.
//! Boundary: centralizes reusable clap argument shapes for SNS commands.

use crate::cli::{
    clap::value_arg,
    common::{
        COLLECTION_MODE_FORCE_REFRESH, collection_help, json_arg, max_pages_arg, page_size_arg,
        source_endpoint_arg,
    },
};
use ic_platform::candid::contract::Principal;
use ic_query::sns::{DEFAULT_SNS_SOURCE_ENDPOINT, SNS_REFRESH_MAX_PAGE_SIZE};

pub(super) fn sns_lookup_input_arg() -> clap::Arg {
    value_arg("input")
        .value_name("id|root-principal")
        .required(true)
        .value_parser(sns_lookup_input_value_parser())
        .help("SNS list id or root canister principal")
}

pub(super) fn sns_root_input_arg() -> clap::Arg {
    value_arg("input")
        .value_name("root-principal")
        .required(true)
        .value_parser(clap::builder::ValueParser::new(|value: &str| {
            Principal::from_text(value)
                .map(|_| value.to_string())
                .map_err(|error| error.to_string())
        }))
        .help("SNS root canister principal identifying the complete collection")
}

pub(super) fn principal_value_parser() -> clap::builder::ValueParser {
    clap::builder::ValueParser::new(|value: &str| {
        Principal::from_text(value).map_err(|err| err.to_string())
    })
}

pub(super) fn sns_neuron_id_value_parser() -> clap::builder::ValueParser {
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

fn sns_lookup_input_value_parser() -> clap::builder::ValueParser {
    clap::builder::ValueParser::new(|value: &str| {
        if value.parse::<usize>().is_ok_and(|id| id > 0) || Principal::from_text(value).is_ok() {
            Ok(value.to_string())
        } else {
            Err("must be a positive SNS list id or root canister principal".to_string())
        }
    })
}

pub(super) fn sns_snapshot_refresh_command(
    command: clap::Command,
    default_page_size: &'static str,
    page_help: &'static str,
    examples: &'static str,
) -> clap::Command {
    command
        .arg(sns_lookup_input_arg())
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance queries"),
        )
        .arg(page_size_arg(default_page_size, SNS_REFRESH_MAX_PAGE_SIZE).help(page_help))
        .arg(
            max_pages_arg()
                .help("Stop before publishing if this page count is reached before API exhaustion"),
        )
        .after_help(collection_help(COLLECTION_MODE_FORCE_REFRESH, examples))
}
