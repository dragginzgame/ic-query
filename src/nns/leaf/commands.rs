use super::model::NnsLeafCommandSpec;
use crate::{
    cli::{
        clap::{flag_arg, parse_matches, passthrough_subcommand, render_help, value_arg},
        common::{format_arg, source_endpoint_arg},
        globals::internal_network_arg,
    },
    duration::parse_duration_seconds,
    nns::NnsCommandError,
    subnet_catalog::MAINNET_NETWORK,
};
use clap::{ArgMatches, Command as ClapCommand};
use std::{ffi::OsString, path::PathBuf};

pub(in crate::nns) const INPUT_ARG: &str = "input";
pub(in crate::nns) const NETWORK_ARG: &str = "network";
pub(in crate::nns) const LOCK_STALE_AFTER_ARG: &str = "lock-stale-after";
pub(in crate::nns) const DRY_RUN_ARG: &str = "dry-run";
pub(in crate::nns) const OUTPUT_ARG: &str = "output";
pub(in crate::nns) const VERBOSE_ARG: &str = "verbose";

const DEFAULT_LOCK_STALE_AFTER: &str = "30m";

pub(in crate::nns) fn parse_leaf_matches<I>(
    command: ClapCommand,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<ArgMatches, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    parse_matches(command, args).map_err(|_| NnsCommandError::Usage(usage()))
}

pub(in crate::nns) fn command(spec: &NnsLeafCommandSpec) -> ClapCommand {
    ClapCommand::new(spec.command_name)
        .bin_name(spec.bin_name)
        .about(spec.about)
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about(spec.list_about),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("info").about(spec.info_about),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("refresh").about(spec.refresh_about),
        ))
}

pub(in crate::nns) fn list_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    ClapCommand::new("list")
        .bin_name(format!("{} list", spec.bin_name))
        .about(spec.list_about)
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(source_endpoint_arg(default_source_endpoint).help(spec.list_source_help))
        .arg(
            flag_arg(VERBOSE_ARG)
                .long(VERBOSE_ARG)
                .help(spec.verbose_help),
        )
        .arg(network_arg())
        .after_help(spec.list_help_after)
}

pub(in crate::nns) fn info_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    ClapCommand::new("info")
        .bin_name(format!("{} info", spec.bin_name))
        .about(spec.info_about)
        .disable_help_flag(true)
        .arg(
            value_arg(INPUT_ARG)
                .value_name(spec.input_value_name)
                .required(true)
                .help(spec.input_help),
        )
        .arg(format_arg())
        .arg(source_endpoint_arg(default_source_endpoint).help(spec.info_source_help))
        .arg(network_arg())
        .after_help(spec.info_help_after)
}

pub(in crate::nns) fn refresh_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name(format!("{} refresh", spec.bin_name))
        .about(spec.refresh_about)
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(source_endpoint_arg(default_source_endpoint).help(spec.refresh_source_help))
        .arg(refresh_lock_stale_after_arg())
        .arg(
            flag_arg(DRY_RUN_ARG)
                .long(DRY_RUN_ARG)
                .help(spec.dry_run_help),
        )
        .arg(output_path_arg().help(spec.output_help))
        .arg(network_arg())
        .after_help(spec.refresh_help_after)
}

pub(in crate::nns) fn usage(spec: &NnsLeafCommandSpec) -> String {
    render_help(command(spec))
}

pub(in crate::nns) fn list_usage(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> String {
    render_help(list_command(spec, default_source_endpoint))
}

pub(in crate::nns) fn info_usage(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> String {
    render_help(info_command(spec, default_source_endpoint))
}

pub(in crate::nns) fn refresh_usage(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> String {
    render_help(refresh_command(spec, default_source_endpoint))
}

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
