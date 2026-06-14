use super::commands::{
    sns_command, sns_info_command, sns_list_command, sns_neurons_cache_command,
    sns_neurons_cache_list_command, sns_neurons_cache_status_command, sns_neurons_command,
    sns_neurons_refresh_command, sns_params_command, sns_proposal_command, sns_proposals_command,
    sns_token_command,
};
use crate::cli::clap::render_help;

pub(in crate::sns::commands) fn usage() -> String {
    render_help(sns_command())
}

pub(in crate::sns::commands) fn sns_list_usage() -> String {
    render_help(sns_list_command())
}

pub(in crate::sns::commands) fn sns_info_usage() -> String {
    render_help(sns_info_command())
}

pub(in crate::sns::commands) fn sns_token_usage() -> String {
    render_help(sns_token_command())
}

pub(in crate::sns::commands) fn sns_params_usage() -> String {
    render_help(sns_params_command())
}

pub(in crate::sns::commands) fn sns_proposal_usage() -> String {
    render_help(sns_proposal_command())
}

pub(in crate::sns::commands) fn sns_proposals_usage() -> String {
    render_help(sns_proposals_command())
}

pub(in crate::sns::commands) fn sns_neurons_usage() -> String {
    render_help(sns_neurons_command())
}

pub(in crate::sns::commands) fn sns_neurons_cache_usage() -> String {
    render_help(sns_neurons_cache_command())
}

pub(in crate::sns::commands) fn sns_neurons_cache_list_usage() -> String {
    render_help(sns_neurons_cache_list_command())
}

pub(in crate::sns::commands) fn sns_neurons_cache_status_usage() -> String {
    render_help(sns_neurons_cache_status_command())
}

pub(in crate::sns::commands) fn sns_neurons_refresh_usage() -> String {
    render_help(sns_neurons_refresh_command())
}
