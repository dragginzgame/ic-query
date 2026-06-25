use super::args::{
    DRY_RUN_ARG, INPUT_ARG, VERBOSE_ARG, network_arg, output_path_arg, refresh_lock_stale_after_arg,
};
use crate::{
    cli::{
        clap::{flag_arg, passthrough_subcommand, value_arg},
        common::{format_arg, source_endpoint_arg},
    },
    nns::leaf::model::NnsLeafCommandSpec,
};
use clap::Command as ClapCommand;

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
