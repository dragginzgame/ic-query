use crate::cli::clap::value_arg;
use candid::Principal;

pub(super) fn sns_lookup_input_arg() -> clap::Arg {
    value_arg("input")
        .value_name("id|root-principal")
        .required(true)
        .value_parser(sns_lookup_input_value_parser())
        .help("SNS list id or root canister principal")
}

pub(super) fn principal_value_parser() -> clap::builder::ValueParser {
    clap::builder::ValueParser::new(|value: &str| {
        Principal::from_text(value).map_err(|err| err.to_string())
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
