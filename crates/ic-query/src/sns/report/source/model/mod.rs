//! Module: sns::report::source::model
//!
//! Responsibility: group SNS source result models.
//! Does not own: live transport, report DTOs, cache IO, or rendering.
//! Boundary: re-exports source-layer data passed from fetchers to builders.

mod canisters;
mod fetch;
mod list;
mod metrics;
mod neurons;
mod proposals;
mod reward;
mod swap;
mod token;
mod upgrade;
mod validation;

pub use canisters::MainnetSnsCanisterInventory;
pub(in crate::sns::report) use canisters::{
    SNS_CANISTER_HEALTH_CALL_TYPE, SNS_CANISTER_HEALTH_METHOD, SNS_CANISTER_INVENTORY_METHOD,
    canonicalize_mainnet_sns_canister_inventory,
};
pub use fetch::SnsSourceRequest;
pub(in crate::sns::report) use list::{
    JoinedMainnetSnsInventory, join_mainnet_sns_inventory, validate_joined_mainnet_sns_inventory,
    validate_mainnet_sns_inventory,
};
pub use list::{MainnetSns, MainnetSnsCanisters, MainnetSnsInventory, MainnetSnsMetadata};
pub use metrics::MainnetSnsMetrics;
pub(in crate::sns::report) use metrics::{
    SNS_METRICS_CALL_TYPE, SNS_METRICS_METHOD, canonicalize_mainnet_sns_metrics, sns_treasury_kind,
};
pub use neurons::{MainnetSnsNeuron, MainnetSnsNeuronPage, MainnetSnsNeurons, SnsNeuronId};
pub(in crate::sns::report) use neurons::{
    sns_neuron_id_from_text, validate_mainnet_sns_neuron, validate_mainnet_sns_neuron_page,
    validate_mainnet_sns_neurons, validate_sns_neuron_rows,
};
pub use proposals::{MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals};
pub use reward::MainnetSnsRewardNeuronPage;
pub(in crate::sns::report) use reward::{
    SnsRewardCollectionState, validate_mainnet_sns_reward_neuron_page,
};
pub use swap::MainnetSnsSwap;
pub(in crate::sns::report) use swap::{
    SNS_SWAP_DERIVED_STATE_METHOD, SNS_SWAP_LIFECYCLE_METHOD, SNS_SWAP_QUERY_COUNT,
    SNS_SWAP_SALE_PARAMETERS_METHOD, canonicalize_mainnet_sns_swap, sns_swap_component_method,
    sns_swap_lifecycle_name,
};
pub use token::MainnetSnsToken;
pub use upgrade::MainnetSnsUpgrade;
pub(in crate::sns::report) use upgrade::{
    SNS_NEXT_VERSION_METHOD, SNS_RUNNING_VERSION_METHOD, SNS_UPGRADE_QUERY_COUNT,
    canonicalize_mainnet_sns_upgrade,
};
