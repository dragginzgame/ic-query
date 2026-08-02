//! Module: sns::commands::spec
//!
//! Responsibility: expose clap command definitions, usage text, and value enums.
//! Does not own: option DTO parsing, command execution, or reports.
//! Boundary: keeps SNS command shape separate from runtime behavior.

mod commands;
mod values;

pub(super) use commands::sns_command;
#[cfg(test)]
pub(super) use commands::{
    sns_canister_command, sns_canister_list_command, sns_info_command, sns_list_command,
    sns_metrics_command, sns_neuron_cache_command, sns_neuron_cache_list_command,
    sns_neuron_cache_status_command, sns_neuron_command, sns_neuron_list_command,
    sns_neuron_refresh_command, sns_params_command, sns_proposal_cache_list_command,
    sns_proposal_cache_status_command, sns_proposal_command, sns_proposal_info_command,
    sns_proposal_list_command, sns_proposal_refresh_command, sns_swap_command, sns_token_command,
    sns_upgrade_command,
};
pub(super) use values::{
    SNS_PROPOSALS_LOCAL_SORT_VALUE_NAME, SnsListSortArg, SnsNeuronsSortArg,
    SnsProposalEligibilityArg, SnsProposalStatusArg, SnsProposalTopicArg, SnsProposalsSortArg,
};
