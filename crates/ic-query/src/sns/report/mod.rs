//! Module: sns::report
//!
//! Responsibility: assemble the reusable SNS query and report surface.
//! Does not own: CLI parsing, process output, or generic cache-file mechanics.
//! Boundary: keeps SNS lookup, live sources, caches, reports, and renderers together.

#[cfg(feature = "host")]
mod assemble;
#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
mod cache_attempt;
#[cfg(feature = "host")]
mod cache_paths;
#[cfg(feature = "host")]
mod cache_refresh;
#[cfg(feature = "host")]
mod cache_status;
#[cfg(feature = "host")]
mod cache_storage;
#[cfg(feature = "host")]
mod cache_summary;
#[cfg(feature = "host")]
mod catalog_cache;
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
mod reward_checkpoint_file;
mod reward_diff;
#[cfg(feature = "host")]
mod source;
mod text;
#[cfg(feature = "host")]
mod view;

#[cfg(all(test, feature = "host"))]
mod tests;

#[cfg(feature = "host")]
use crate::hex::hex_bytes;
#[cfg(all(test, feature = "host"))]
use crate::icrc::{
    IcrcMetadataValueKind,
    ledger::{IcrcMetadataValue, metadata_row},
};
#[cfg(all(test, feature = "host"))]
use crate::subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs};
#[cfg(all(test, feature = "host"))]
use neurons_cache::{
    SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, SNS_NEURONS_CACHE_SCHEMA_VERSION,
    SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
};
#[cfg(all(test, feature = "host"))]
pub(in crate::sns::report) use source::validate_mainnet_sns_reward_neuron_page;
#[cfg(feature = "host")]
pub(in crate::sns::report) use source::{
    JoinedMainnetSnsInventory, SNS_SWAP_QUERY_COUNT, SNS_UPGRADE_QUERY_COUNT,
    SnsRewardCollectionState,
};

#[cfg(feature = "host")]
pub use build::{
    build_sns_canister_report, build_sns_canister_report_with_source, build_sns_info_report,
    build_sns_info_report_with_source, build_sns_list_report, build_sns_list_report_with_source,
    build_sns_metrics_report, build_sns_metrics_report_with_source, build_sns_neuron_detail_report,
    build_sns_neuron_detail_report_with_source, build_sns_neurons_report,
    build_sns_neurons_report_with_source, build_sns_params_report,
    build_sns_params_report_with_source, build_sns_proposal_report,
    build_sns_proposal_report_with_source, build_sns_proposals_report,
    build_sns_proposals_report_with_progress, build_sns_proposals_report_with_source,
    build_sns_reward_checkpoint_report, build_sns_reward_checkpoint_report_with_source,
    build_sns_swap_report, build_sns_swap_report_with_source, build_sns_token_report,
    build_sns_token_report_with_source, build_sns_upgrade_report,
    build_sns_upgrade_report_with_source,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use cache_summary::{
    build_sns_cache_list_report, find_sns_cache_summary_by_id, load_sns_cache_summary_at,
    parse_sns_root_canister_input,
};
#[cfg(feature = "host")]
pub use catalog_cache::{
    DEFAULT_SNS_CATALOG_REFRESH_LOCK_STALE_SECONDS, DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS,
    SnsCatalogCacheRequest, SnsCatalogRefreshReport, SnsCatalogRefreshRequest,
    build_sns_list_report_from_cache, build_sns_list_report_from_cache_or_refresh,
    build_sns_list_report_from_cache_or_refresh_with_source, refresh_sns_catalog,
    refresh_sns_catalog_with_source, sns_catalog_cache_path, sns_catalog_refresh_lock_path,
    sns_catalog_refresh_report_text,
};
#[cfg(feature = "host")]
pub use live::LiveSnsSource;
pub use model::{
    DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS, MAX_SNS_METRICS_TIME_WINDOW_SECONDS,
    SnsCanisterCallType, SnsCanisterGap, SnsCanisterGapKind, SnsCanisterReport, SnsCanisterRole,
    SnsCanisterRow, SnsCanisterStatus, SnsCustomProposalCriticality, SnsDefaultFollowees,
    SnsDefaultFolloweesRow, SnsGovernanceParameters, SnsInfoReport, SnsListReport, SnsListRequest,
    SnsListRow, SnsListSort, SnsLookupRequest, SnsMaturityDisbursementRow, SnsMetricsReport,
    SnsMetricsRequest, SnsNeuronAccount, SnsNeuronDetail, SnsNeuronDetailReport,
    SnsNeuronDissolveState, SnsNeuronFolloweeRow, SnsNeuronFolloweesRow, SnsNeuronPermissionList,
    SnsNeuronPermissionRow, SnsNeuronPermissionValue, SnsNeuronRow, SnsNeuronTopicFolloweesRow,
    SnsParamsReport, SnsPendingUpgrade, SnsPolicyObservationStatus, SnsProposalBallotRow,
    SnsProposalDecisionState, SnsProposalEligibilityFilter, SnsProposalFailureReason,
    SnsProposalReport, SnsProposalRequest, SnsProposalRow, SnsProposalSortDirection,
    SnsProposalStatusFilter, SnsProposalTally, SnsProposalTopicFilter, SnsProposalsReport,
    SnsProposalsRequest, SnsProposalsSort, SnsRewardAllocationStatus, SnsRewardCheckpointReport,
    SnsRewardCheckpointRow, SnsRewardCheckpointValidationError, SnsRewardCollectionStatus,
    SnsRewardDiffCheckpointRef, SnsRewardDiffInvalidReason, SnsRewardDiffInvalidReasonKind,
    SnsRewardDiffReport, SnsRewardDiffRow, SnsRewardEvent, SnsRewardProposalId,
    SnsRunningVersionResponse, SnsSwapComponent, SnsSwapDerivedState, SnsSwapLifecycle,
    SnsSwapNeuronBasketConstructionParameters, SnsSwapQueryGap, SnsSwapReport,
    SnsSwapSaleParameters, SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow,
    SnsTreasuryKind, SnsTreasuryMetricRow, SnsUpgradeQueryGap, SnsUpgradeReport, SnsVersion,
    SnsVotingPowerMetrics, SnsVotingRewardsParameters, sns_neuron_permission_name,
    validate_sns_reward_checkpoint_report,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use model::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(all(test, feature = "host"))]
pub(in crate::sns::report) use model::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
#[cfg(feature = "host")]
pub use model::{
    SnsCacheListReport, SnsCacheListRequest, SnsCacheStatusReport, SnsCacheStatusRequest,
    SnsCacheSummary, SnsHostError, SnsNeuronRequest, SnsNeuronsRefreshReport,
    SnsNeuronsRefreshRequest, SnsNeuronsReport, SnsNeuronsRequest, SnsNeuronsSort,
    SnsProposalsRefreshReport, SnsProposalsRefreshRequest, SnsRefreshAttemptStatus,
    SnsRewardCheckpointRequest,
};
pub(in crate::sns::report) use model::{
    SnsRewardCheckpointSummary, recompute_reward_checkpoint_summary,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use model::{
    validate_sns_reward_checkpoint_parameter_evidence, validate_sns_reward_event_evidence,
    validate_sns_reward_running_version_evidence,
};
#[cfg(feature = "host")]
pub use neurons_cache::{
    DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS, build_sns_neurons_cache_list_report,
    build_sns_neurons_cache_status_report, refresh_sns_neurons_cache,
    refresh_sns_neurons_cache_with_progress, refresh_sns_neurons_cache_with_source,
    sns_neurons_cache_path, sns_neurons_refresh_attempt_path, sns_neurons_refresh_lock_path,
};
#[cfg(feature = "host")]
pub use proposals_cache::{
    DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS, build_sns_proposals_cache_list_report,
    build_sns_proposals_cache_status_report, refresh_sns_proposals_cache,
    refresh_sns_proposals_cache_with_progress, refresh_sns_proposals_cache_with_source,
    sns_proposals_cache_path, sns_proposals_refresh_attempt_path, sns_proposals_refresh_lock_path,
};
#[cfg(feature = "host")]
pub use reward_checkpoint_file::{
    build_sns_reward_diff_report_from_paths, load_sns_reward_checkpoint,
};
pub use reward_diff::build_sns_reward_diff_report;
#[cfg(feature = "host")]
pub use source::{
    MainnetSns, MainnetSnsCanisterInventory, MainnetSnsCanisters, MainnetSnsInventory,
    MainnetSnsMetadata, MainnetSnsMetrics, MainnetSnsNeuron, MainnetSnsNeuronPage,
    MainnetSnsNeurons, MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals,
    MainnetSnsRewardNeuronPage, MainnetSnsSwap, MainnetSnsToken, MainnetSnsUpgrade,
    SnsCanisterSource, SnsDiscoverySource, SnsMetricsSource, SnsNeuronId, SnsNeuronSource,
    SnsNeuronsSource, SnsParamsSource, SnsProposalSource, SnsProposalsSource, SnsRewardSource,
    SnsSourceRequest, SnsSwapSource, SnsTokenSource, SnsUpgradeSource,
};
pub use text::{
    sns_canister_report_text, sns_info_report_text, sns_list_report_text, sns_metrics_report_text,
    sns_neuron_detail_report_text, sns_params_report_text, sns_proposal_report_text,
    sns_proposals_report_text, sns_reward_checkpoint_report_text, sns_reward_diff_report_text,
    sns_swap_report_text, sns_token_report_text, sns_upgrade_report_text,
};
#[cfg(feature = "host")]
pub use text::{
    sns_neurons_cache_list_report_text, sns_neurons_cache_status_report_text,
    sns_neurons_refresh_report_text, sns_neurons_report_text, sns_proposals_cache_list_report_text,
    sns_proposals_cache_status_report_text, sns_proposals_refresh_report_text,
};

pub const DEFAULT_SNS_SOURCE_ENDPOINT: &str = "https://icp-api.io";
pub const MAINNET_SNS_WASM_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";
#[cfg(feature = "host")]
/// Largest page size accepted by an SNS refresh request.
pub const SNS_REFRESH_MAX_PAGE_SIZE: u32 = 100;

#[cfg(feature = "host")]
const SNS_LIST_REPORT_SCHEMA_VERSION: u32 = 2;
#[cfg(feature = "host")]
const SNS_CANISTER_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_METRICS_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_PARAMS_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_SWAP_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_UPGRADE_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_PROPOSAL_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_PROPOSALS_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const SNS_NEURONS_REPORT_SCHEMA_VERSION: u32 = 2;
#[cfg(feature = "host")]
const SNS_NEURON_DETAIL_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_REWARD_CHECKPOINT_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_REWARD_CHECKPOINT_MAX_NEURONS: u64 = 200_000;
const SNS_REWARD_CHECKPOINT_PAGE_SIZE: u32 = 100;
const SNS_REWARD_DIFF_REPORT_SCHEMA_VERSION: u32 = 1;
const COMPACT_PRINCIPAL_CHARS: usize = 5;
#[cfg(feature = "host")]
const SNS_METADATA_CONCURRENCY: usize = 16;

#[cfg(feature = "host")]
pub(in crate::sns::report) fn enforce_mainnet_network(network: &str) -> Result<(), SnsHostError> {
    crate::network::enforce_mainnet_network_with(network, |network| {
        SnsHostError::UnsupportedNetwork { network }
    })
}

pub(super) fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}
