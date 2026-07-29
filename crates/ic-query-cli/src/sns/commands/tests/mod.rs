use super::{
    error::SnsCommandError,
    options::{
        SnsListOptions, SnsLookupOptions, SnsNeuronsCacheListOptions, SnsNeuronsCacheStatusOptions,
        SnsNeuronsOptions, SnsNeuronsRefreshOptions, SnsProposalOptions,
        SnsProposalsCacheListOptions, SnsProposalsCacheStatusOptions, SnsProposalsOptions,
        SnsProposalsRefreshOptions,
    },
    spec::{
        SnsListSortArg, SnsNeuronsSortArg, SnsProposalEligibilityArg, SnsProposalStatusArg,
        SnsProposalTopicArg, SnsProposalsSortArg, sns_info_command, sns_info_usage, sns_list_usage,
        sns_neuron_cache_list_usage, sns_neuron_cache_status_usage, sns_neuron_cache_usage,
        sns_neuron_list_usage, sns_neuron_refresh_usage, sns_neuron_usage, sns_params_command,
        sns_params_usage, sns_proposal_cache_list_usage, sns_proposal_cache_status_usage,
        sns_proposal_info_usage, sns_proposal_list_usage, sns_proposal_refresh_usage,
        sns_proposal_usage, sns_token_command, sns_token_usage, usage,
    },
};
use crate::{cli::common::OutputFormat, test_support::assert_snapshot};
use ic_query::sns::{DEFAULT_SNS_SOURCE_ENDPOINT, SnsProposalSortDirection};
use std::ffi::OsString;

mod invalid;
mod list;
mod lookup;
mod neurons;
mod proposals;
mod usage;
