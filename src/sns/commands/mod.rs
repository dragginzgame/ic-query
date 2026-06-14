mod error;
mod options;
mod run;
mod spec;

pub use error::SnsCommandError;
pub use run::run;

#[cfg(test)]
use crate::{cli::common::OutputFormat, sns::report::DEFAULT_SNS_SOURCE_ENDPOINT};
#[cfg(test)]
use options::{
    SnsListOptions, SnsLookupOptions, SnsNeuronsCacheListOptions, SnsNeuronsCacheStatusOptions,
    SnsNeuronsOptions, SnsNeuronsRefreshOptions, SnsProposalOptions, SnsProposalsOptions,
};
#[cfg(test)]
use spec::{
    SnsListSortArg, SnsNeuronsSortArg, SnsProposalStatusArg, sns_info_command, sns_info_usage,
    sns_list_usage, sns_neurons_cache_list_usage, sns_neurons_cache_status_usage,
    sns_neurons_cache_usage, sns_neurons_refresh_usage, sns_neurons_usage, sns_params_command,
    sns_params_usage, sns_proposal_usage, sns_proposals_usage, sns_token_command, sns_token_usage,
    usage,
};
#[cfg(test)]
use std::ffi::OsString;

#[cfg(test)]
mod tests;
