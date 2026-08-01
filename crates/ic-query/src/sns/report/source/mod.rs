//! Module: sns::report::source
//!
//! Responsibility: group SNS source models and source traits.
//! Does not own: live transport implementations, cache IO, report assembly, or rendering.
//! Boundary: exposes source-layer contracts used by report builders and tests.

mod model;
mod traits;

pub(in crate::sns::report) use model::{
    JoinedMainnetSnsInventory, SNS_CANISTER_HEALTH_CALL_TYPE, SNS_CANISTER_HEALTH_METHOD,
    SNS_CANISTER_INVENTORY_METHOD, SNS_METRICS_CALL_TYPE, SNS_METRICS_METHOD,
    SNS_NEXT_VERSION_METHOD, SNS_RUNNING_VERSION_METHOD, SNS_SWAP_DERIVED_STATE_METHOD,
    SNS_SWAP_LIFECYCLE_METHOD, SNS_SWAP_QUERY_COUNT, SNS_SWAP_SALE_PARAMETERS_METHOD,
    SNS_UPGRADE_QUERY_COUNT, canonicalize_mainnet_sns_canister_inventory,
    canonicalize_mainnet_sns_metrics, canonicalize_mainnet_sns_swap,
    canonicalize_mainnet_sns_upgrade, join_mainnet_sns_inventory, sns_swap_component_method,
    sns_swap_lifecycle_name, sns_treasury_kind, validate_mainnet_sns_inventory,
    validate_mainnet_sns_neuron_page, validate_mainnet_sns_neurons, validate_sns_neuron_rows,
};
pub use model::{
    MainnetSns, MainnetSnsCanisterInventory, MainnetSnsCanisters, MainnetSnsInventory,
    MainnetSnsMetadata, MainnetSnsMetrics, MainnetSnsNeuronPage, MainnetSnsNeurons,
    MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals, MainnetSnsSwap,
    MainnetSnsToken, MainnetSnsUpgrade, SnsNeuronId, SnsSourceRequest,
};
pub use traits::{
    SnsCanisterSource, SnsDiscoverySource, SnsMetricsSource, SnsNeuronsSource, SnsParamsSource,
    SnsProposalSource, SnsProposalsSource, SnsSwapSource, SnsTokenSource, SnsUpgradeSource,
};
