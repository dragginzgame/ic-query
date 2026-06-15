use crate::{hex::hex_bytes, ic_registry::DEFAULT_MAINNET_ENDPOINT};
use lookup::enforce_mainnet_network;
pub use model::*;
use source::{
    MainnetSns, MainnetSnsCanisters, MainnetSnsList, MainnetSnsNeuronPage, MainnetSnsNeurons,
    MainnetSnsProposal, MainnetSnsProposals, MainnetSnsToken, SnsFetchRequest, SnsListSource,
    SnsNeuronId, SnsNeuronsSource, SnsParamsSource, SnsProposalSource, SnsProposalsSource,
    SnsTokenSource,
};

mod assemble;
mod build;
mod live;
mod lookup;
mod model;
mod neurons_cache;
mod source;
mod text;

pub use build::{
    build_sns_info_report, build_sns_list_report, build_sns_neurons_report,
    build_sns_params_report, build_sns_proposal_report, build_sns_proposals_report,
    build_sns_token_report,
};
#[cfg(test)]
use neurons_cache::{
    SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, SNS_NEURONS_CACHE_SCHEMA_VERSION,
    SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION, refresh_sns_neurons_cache_with_source,
    sns_neurons_cache_path, sns_neurons_refresh_attempt_path, sns_neurons_refresh_lock_path,
};
pub use neurons_cache::{
    build_sns_neurons_cache_list_report, build_sns_neurons_cache_status_report,
    refresh_sns_neurons_cache,
};
pub use text::{
    sns_info_report_text, sns_list_report_text, sns_neurons_cache_list_report_text,
    sns_neurons_cache_status_report_text, sns_neurons_refresh_report_text, sns_neurons_report_text,
    sns_params_report_text, sns_proposal_report_text, sns_proposals_report_text,
    sns_token_report_text,
};

#[cfg(test)]
use live::{IcrcMetadataValue, metadata_row};

#[cfg(test)]
use crate::subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs};

pub const DEFAULT_SNS_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const MAINNET_SNS_WASM_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";

const SNS_LIST_REPORT_SCHEMA_VERSION: u32 = 3;
const SNS_INFO_REPORT_SCHEMA_VERSION: u32 = 2;
const SNS_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_PARAMS_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_PROPOSAL_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_PROPOSALS_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_NEURONS_REPORT_SCHEMA_VERSION: u32 = 1;
const COMPACT_PRINCIPAL_CHARS: usize = 5;
const SNS_TOKEN_LOGO_METADATA_KEY: &str = "icrc1:logo";
const SNS_METADATA_CONCURRENCY: usize = 16;

pub(super) fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}

#[cfg(test)]
use build::{
    build_sns_info_report_with_source, build_sns_list_report_with_source,
    build_sns_neurons_report_with_source, build_sns_params_report_with_source,
    build_sns_proposal_report_with_source, build_sns_proposals_report_with_source,
    build_sns_token_report_with_source,
};

#[cfg(test)]
mod tests;
