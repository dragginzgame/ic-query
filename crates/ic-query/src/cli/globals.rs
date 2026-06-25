use crate::cli::clap::value_arg;
use clap::Arg;

const INTERNAL_NETWORK_OPTION: &str = "__icq-network";

pub fn internal_network_arg() -> Arg {
    value_arg("network")
        .long(INTERNAL_NETWORK_OPTION)
        .hide(true)
}
