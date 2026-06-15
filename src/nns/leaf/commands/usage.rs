use super::build::{command, info_command, list_command, refresh_command};
use crate::{cli::clap::render_help, nns::leaf::model::NnsLeafCommandSpec};

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
