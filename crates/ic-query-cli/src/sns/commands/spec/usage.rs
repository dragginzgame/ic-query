//! Module: sns::commands::spec::usage
//!
//! Responsibility: render help text for SNS command specs.
//! Does not own: command execution, option parsing, or report text.
//! Boundary: converts clap command definitions into human-facing usage text.

use crate::{
    cli::clap::render_help,
    sns::commands::spec::commands::{
        sns_canister_command, sns_canister_list_command, sns_command, sns_info_command,
        sns_list_command, sns_metrics_command, sns_neuron_cache_command,
        sns_neuron_cache_list_command, sns_neuron_cache_status_command, sns_neuron_command,
        sns_neuron_list_command, sns_neuron_refresh_command, sns_params_command,
        sns_proposal_cache_list_command, sns_proposal_cache_status_command, sns_proposal_command,
        sns_proposal_info_command, sns_proposal_list_command, sns_proposal_refresh_command,
        sns_swap_command, sns_token_command, sns_upgrade_command,
    },
};

pub(in crate::sns::commands) fn usage() -> String {
    render_help(sns_command())
}

pub(in crate::sns::commands) fn sns_list_usage() -> String {
    render_help(sns_list_command())
}

pub(in crate::sns::commands) fn sns_info_usage() -> String {
    render_help(sns_info_command())
}

pub(in crate::sns::commands) fn sns_metrics_usage() -> String {
    render_help(sns_metrics_command())
}

pub(in crate::sns::commands) fn sns_token_usage() -> String {
    render_help(sns_token_command())
}

pub(in crate::sns::commands) fn sns_params_usage() -> String {
    render_help(sns_params_command())
}

pub(in crate::sns::commands) fn sns_swap_usage() -> String {
    render_help(sns_swap_command())
}

pub(in crate::sns::commands) fn sns_upgrade_usage() -> String {
    render_help(sns_upgrade_command())
}

pub(in crate::sns::commands) fn sns_canister_usage() -> String {
    render_help(sns_canister_command())
}

pub(in crate::sns::commands) fn sns_canister_list_usage() -> String {
    render_help(sns_canister_list_command())
}

pub(in crate::sns::commands) fn sns_proposal_usage() -> String {
    render_help(sns_proposal_command())
}

pub(in crate::sns::commands) fn sns_proposal_list_usage() -> String {
    render_help(sns_proposal_list_command())
}

pub(in crate::sns::commands) fn sns_proposal_info_usage() -> String {
    render_help(sns_proposal_info_command())
}

pub(in crate::sns::commands) fn sns_proposal_cache_list_usage() -> String {
    render_help(sns_proposal_cache_list_command())
}

pub(in crate::sns::commands) fn sns_proposal_cache_status_usage() -> String {
    render_help(sns_proposal_cache_status_command())
}

pub(in crate::sns::commands) fn sns_proposal_refresh_usage() -> String {
    render_help(sns_proposal_refresh_command())
}

pub(in crate::sns::commands) fn sns_neuron_usage() -> String {
    render_help(sns_neuron_command())
}

pub(in crate::sns::commands) fn sns_neuron_list_usage() -> String {
    render_help(sns_neuron_list_command())
}

pub(in crate::sns::commands) fn sns_neuron_cache_usage() -> String {
    render_help(sns_neuron_cache_command())
}

pub(in crate::sns::commands) fn sns_neuron_cache_list_usage() -> String {
    render_help(sns_neuron_cache_list_command())
}

pub(in crate::sns::commands) fn sns_neuron_cache_status_usage() -> String {
    render_help(sns_neuron_cache_status_command())
}

pub(in crate::sns::commands) fn sns_neuron_refresh_usage() -> String {
    render_help(sns_neuron_refresh_command())
}
