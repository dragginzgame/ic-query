use crate::cli::clap::with_options;
use crate::{
    cli::common::{
        COLLECTION_MODE_CACHE_REFRESH_MISSING, COLLECTION_MODE_FORCE_REFRESH, collection_help,
    },
    nns::leaf::{
        model::NnsLeafCommandSpec,
        options::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions},
    },
};
use clap::Command as ClapCommand;

pub(in crate::nns) fn command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    command_with_list(
        spec,
        default_source_endpoint,
        list_command(spec, default_source_endpoint),
    )
}

pub(in crate::nns) fn command_with_list(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
    list: ClapCommand,
) -> ClapCommand {
    ClapCommand::new(spec.command_name)
        .about(spec.about)
        .subcommand(list)
        .subcommand(info_command(spec, default_source_endpoint))
        .subcommand(refresh_command(spec, default_source_endpoint))
}

pub(in crate::nns) fn list_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    with_options::<NnsLeafListOptions>(ClapCommand::new("list").about(spec.list_about))
        .mut_arg("source-endpoint", |arg| {
            arg.default_value(default_source_endpoint)
                .help(spec.read_source_help)
        })
        .mut_arg("verbose", |arg| arg.help(spec.verbose_help))
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_REFRESH_MISSING,
            spec.list_help_after,
        ))
}

pub(in crate::nns) fn info_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    with_options::<NnsLeafInfoOptions>(ClapCommand::new("info").about(spec.info_about))
        .mut_arg("input", |arg| {
            arg.value_name(spec.input_value_name).help(spec.input_help)
        })
        .mut_arg("source-endpoint", |arg| {
            arg.default_value(default_source_endpoint)
                .help(spec.read_source_help)
        })
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_REFRESH_MISSING,
            spec.info_help_after,
        ))
}

pub(in crate::nns) fn refresh_command(
    spec: &NnsLeafCommandSpec,
    default_source_endpoint: &'static str,
) -> ClapCommand {
    with_options::<NnsLeafRefreshOptions>(ClapCommand::new("refresh").about(spec.refresh_about))
        .mut_arg("source-endpoint", |arg| {
            arg.default_value(default_source_endpoint)
                .help(spec.refresh_source_help)
        })
        .mut_arg("dry-run", |arg| {
            arg.help(format!(
                "Fetch and validate without replacing the cached {} report",
                spec.command_name,
            ))
        })
        .mut_arg("output", |arg| {
            arg.help(format!(
                "Also write the fetched {} JSON to this path",
                spec.command_name,
            ))
        })
        .after_help(collection_help(
            COLLECTION_MODE_FORCE_REFRESH,
            spec.refresh_help_after,
        ))
}
