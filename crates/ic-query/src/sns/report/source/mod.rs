//! Module: sns::report::source
//!
//! Responsibility: group SNS source models and source traits.
//! Does not own: live transport implementations, cache IO, report assembly, or rendering.
//! Boundary: exposes source-layer contracts used by report builders and tests.

mod model;
mod traits;

pub(in crate::sns::report) use model::{
    JoinedMainnetSnsInventory, SNS_CANISTER_HEALTH_CALL_TYPE, SNS_METRICS_CALL_TYPE,
    SNS_SWAP_QUERY_COUNT, SNS_UPGRADE_QUERY_COUNT, SnsRewardCollectionState,
    canonicalize_mainnet_sns_canister_inventory, canonicalize_mainnet_sns_metrics,
    canonicalize_mainnet_sns_swap, canonicalize_mainnet_sns_upgrade, join_mainnet_sns_inventory,
    join_mainnet_sns_lifecycles, sns_neuron_id_from_text, sns_swap_component_method,
    sns_swap_lifecycle_name, sns_treasury_kind, validate_joined_mainnet_sns_catalog,
    validate_mainnet_sns_inventory, validate_mainnet_sns_neuron, validate_mainnet_sns_neuron_page,
    validate_mainnet_sns_neurons, validate_mainnet_sns_proposal,
    validate_mainnet_sns_proposal_page, validate_mainnet_sns_proposals,
    validate_mainnet_sns_reward_neuron_page, validate_sns_neuron_rows, validate_sns_proposal_rows,
};
pub use model::{
    MainnetSns, MainnetSnsCanisterInventory, MainnetSnsCanisters, MainnetSnsInventory,
    MainnetSnsLifecycle, MainnetSnsMetadata, MainnetSnsMetrics, MainnetSnsNeuron,
    MainnetSnsNeuronPage, MainnetSnsNeurons, MainnetSnsProposal, MainnetSnsProposalPage,
    MainnetSnsProposals, MainnetSnsRewardNeuronPage, MainnetSnsSwap, MainnetSnsToken,
    MainnetSnsUpgrade, SnsNeuronId, SnsSourceRequest,
};
pub use traits::{
    SnsCanisterSource, SnsCatalogSource, SnsDiscoverySource, SnsMetricsSource, SnsNeuronSource,
    SnsNeuronsSource, SnsParamsSource, SnsProposalSource, SnsProposalsSource, SnsRewardSource,
    SnsSwapSource, SnsTokenSource, SnsUpgradeSource,
};
