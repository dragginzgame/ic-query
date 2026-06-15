use crate::{
    cli::{clap::value_arg, globals::internal_network_arg},
    duration::parse_duration_seconds,
    subnet_catalog::MAINNET_NETWORK,
};
use std::path::PathBuf;

pub(in crate::nns) const INPUT_ARG: &str = "input";
pub(in crate::nns) const NETWORK_ARG: &str = "network";
pub(in crate::nns) const LOCK_STALE_AFTER_ARG: &str = "lock-stale-after";
pub(in crate::nns) const DRY_RUN_ARG: &str = "dry-run";
pub(in crate::nns) const OUTPUT_ARG: &str = "output";
pub(in crate::nns) const VERBOSE_ARG: &str = "verbose";

const DEFAULT_LOCK_STALE_AFTER: &str = "30m";

pub(in crate::nns) fn network_arg() -> clap::Arg {
    internal_network_arg().default_value(MAINNET_NETWORK)
}

pub(in crate::nns) fn refresh_lock_stale_after_arg() -> clap::Arg {
    value_arg(LOCK_STALE_AFTER_ARG)
        .long(LOCK_STALE_AFTER_ARG)
        .value_name("duration")
        .default_value(DEFAULT_LOCK_STALE_AFTER)
        .value_parser(clap::builder::ValueParser::new(
            parse_refresh_lock_stale_after,
        ))
        .help("Treat an existing refresh lock as stale after this duration; defaults to 30m")
}

pub(in crate::nns) fn output_path_arg() -> clap::Arg {
    value_arg(OUTPUT_ARG)
        .long(OUTPUT_ARG)
        .value_name("path")
        .value_parser(clap::value_parser!(PathBuf))
}

fn parse_refresh_lock_stale_after(value: &str) -> Result<u64, String> {
    parse_duration_seconds(value).map_err(|err| err.to_string())
}
