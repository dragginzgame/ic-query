use super::{
    info::info_command, list::list_command, refresh::refresh_command, root::subnet_command,
};
use crate::cli::clap::render_help;

pub(in crate::nns) fn subnet_usage() -> String {
    render_help(subnet_command())
}

pub(in crate::nns) fn list_usage() -> String {
    render_help(list_command())
}

pub(in crate::nns) fn info_usage() -> String {
    render_help(info_command())
}

pub(in crate::nns) fn refresh_usage() -> String {
    render_help(refresh_command())
}
