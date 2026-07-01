#[cfg(feature = "host")]
use crate::hex::hex_bytes;
#[cfg(feature = "host")]
use lookup::enforce_mainnet_network;
#[cfg(feature = "host")]
pub(in crate::sns::report) use model::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN, SNS_PROPOSAL_STATUS_ADOPTED_CODE,
    SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(all(test, feature = "host"))]
pub(in crate::sns::report) use model::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
pub use model::{
    SnsCustomProposalCriticality, SnsGovernanceParameters, SnsInfoReport, SnsInfoRequest,
    SnsListReport, SnsListRequest, SnsListRow, SnsListSort, SnsLookupRequest,
    SnsNeuronPermissionList, SnsParamsReport, SnsParamsRequest, SnsProposalBallotRow,
    SnsProposalEligibilityFilter, SnsProposalFailureReason, SnsProposalReport, SnsProposalRequest,
    SnsProposalRow, SnsProposalSortDirection, SnsProposalStatusFilter, SnsProposalTally,
    SnsProposalTopicFilter, SnsProposalsReport, SnsProposalsRequest, SnsProposalsSort,
    SnsTokenMetadataRow, SnsTokenReport, SnsTokenRequest, SnsTokenStandardRow,
    SnsVotingRewardsParameters,
};
#[cfg(feature = "host")]
pub use model::{
    SnsHostError, SnsNeuronRow, SnsNeuronsCacheListReport, SnsNeuronsCacheListRequest,
    SnsNeuronsCacheStatusReport, SnsNeuronsCacheStatusRequest, SnsNeuronsCacheSummary,
    SnsNeuronsRefreshAttemptStatus, SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest,
    SnsNeuronsReport, SnsNeuronsRequest, SnsNeuronsSort, SnsProposalsCacheListReport,
    SnsProposalsCacheListRequest, SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport,
    SnsProposalsRefreshRequest,
};
#[cfg(feature = "host")]
pub use source::{
    MainnetSns, MainnetSnsList, MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals,
    MainnetSnsToken, SnsListSource, SnsParamsSource, SnsProposalSource, SnsProposalsSource,
    SnsSourceRequest, SnsTokenSource,
};
#[cfg(feature = "host")]
use source::{
    MainnetSnsCanisters, MainnetSnsNeuronPage, MainnetSnsNeurons, SnsFetchRequest, SnsNeuronId,
    SnsNeuronsSource,
};

#[cfg(feature = "host")]
mod assemble;
#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
mod cache_attempt;
#[cfg(feature = "host")]
mod cache_paths;
#[cfg(feature = "host")]
mod cache_status;
#[cfg(feature = "host")]
mod cache_storage;
#[cfg(feature = "host")]
mod cache_summary;
#[cfg(feature = "host")]
mod live;
#[cfg(feature = "host")]
mod lookup;
mod model;
#[cfg(feature = "host")]
mod neurons_cache;
#[cfg(feature = "host")]
mod proposals_cache;
#[cfg(feature = "host")]
mod source;
mod text;
#[cfg(feature = "host")]
mod view;

#[cfg(feature = "host")]
pub use build::{
    build_sns_info_report, build_sns_info_report_with_source, build_sns_list_report,
    build_sns_list_report_with_source, build_sns_neurons_report, build_sns_params_report,
    build_sns_params_report_with_source, build_sns_proposal_report,
    build_sns_proposal_report_with_source, build_sns_proposals_report,
    build_sns_proposals_report_with_source, build_sns_token_report,
    build_sns_token_report_with_source,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use cache_summary::{
    SnsCacheListFamily, SnsCacheSummarySortKey, build_sns_cache_list_lookup,
    find_valid_sns_cache_summary_by_id, invalid_sns_cache_summary_fields,
    parse_sns_root_canister_input,
};
#[cfg(feature = "host")]
pub use live::LiveSnsSource;
#[cfg(feature = "host")]
pub use neurons_cache::{
    DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS, build_sns_neurons_cache_list_report,
    build_sns_neurons_cache_status_report, refresh_sns_neurons_cache, sns_neurons_cache_path,
    sns_neurons_refresh_attempt_path, sns_neurons_refresh_lock_path,
};
#[cfg(all(test, feature = "host"))]
use neurons_cache::{
    SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, SNS_NEURONS_CACHE_SCHEMA_VERSION,
    SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION, refresh_sns_neurons_cache_with_source,
};
#[cfg(feature = "host")]
pub use proposals_cache::{
    DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS, build_sns_proposals_cache_list_report,
    build_sns_proposals_cache_status_report, refresh_sns_proposals_cache,
    refresh_sns_proposals_cache_with_source, sns_proposals_cache_path,
    sns_proposals_refresh_attempt_path, sns_proposals_refresh_lock_path,
};
pub use text::{
    sns_info_report_text, sns_list_report_text, sns_params_report_text, sns_proposal_report_text,
    sns_proposals_report_text, sns_token_report_text,
};
#[cfg(feature = "host")]
pub use text::{
    sns_neurons_cache_list_report_text, sns_neurons_cache_status_report_text,
    sns_neurons_refresh_report_text, sns_neurons_report_text, sns_proposals_cache_list_report_text,
    sns_proposals_cache_status_report_text, sns_proposals_refresh_report_text,
};

#[cfg(all(test, feature = "host"))]
use crate::icrc::ledger::{IcrcMetadataValue, metadata_row};

#[cfg(all(test, feature = "host"))]
use crate::subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs};

pub const DEFAULT_SNS_SOURCE_ENDPOINT: &str = "https://icp-api.io";
pub const MAINNET_SNS_WASM_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";

#[cfg(feature = "host")]
const SNS_LIST_REPORT_SCHEMA_VERSION: u32 = 3;
#[cfg(feature = "host")]
const SNS_INFO_REPORT_SCHEMA_VERSION: u32 = 2;
#[cfg(feature = "host")]
const SNS_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_PARAMS_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_PROPOSAL_REPORT_SCHEMA_VERSION: u32 = 5;
#[cfg(feature = "host")]
const SNS_PROPOSALS_REPORT_SCHEMA_VERSION: u32 = 10;
#[cfg(feature = "host")]
const SNS_NEURONS_REPORT_SCHEMA_VERSION: u32 = 1;
const COMPACT_PRINCIPAL_CHARS: usize = 5;
#[cfg(all(test, feature = "host"))]
const SNS_TOKEN_LOGO_METADATA_KEY: &str = "icrc1:logo";
#[cfg(feature = "host")]
const SNS_METADATA_CONCURRENCY: usize = 16;

pub(super) fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}

#[cfg(all(test, feature = "host"))]
use build::build_sns_neurons_report_with_source;

#[cfg(all(test, feature = "host"))]
mod tests;
