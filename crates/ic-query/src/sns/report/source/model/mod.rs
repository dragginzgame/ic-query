//! Module: sns::report::source::model
//!
//! Responsibility: group SNS source result models.
//! Does not own: live transport, report DTOs, cache IO, or rendering.
//! Boundary: re-exports source-layer data passed from fetchers to builders.

mod canisters;
mod fetch;
pub(in crate::sns::report) mod list;
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
    SNS_CANISTER_HEALTH_CALL_TYPE, canonicalize_mainnet_sns_canister_inventory,
};
pub use fetch::SnsSourceRequest;
pub(in crate::sns::report) use list::{
    JoinedMainnetSnsInventory, join_mainnet_sns_inventory, join_mainnet_sns_lifecycles,
    validate_joined_mainnet_sns_catalog, validate_mainnet_sns_inventory,
};
pub use list::{MainnetSns, MainnetSnsInventory, MainnetSnsLifecycle, MainnetSnsMetadata};
pub(in crate::sns::report) use metrics::{SNS_METRICS_CALL_TYPE, canonicalize_mainnet_sns_metrics};
pub use neurons::{MainnetSnsNeuronPage, SnsNeuronId};
pub(in crate::sns::report) use neurons::{
    sns_neuron_id_from_text, validate_mainnet_sns_neuron, validate_mainnet_sns_neuron_page,
    validate_mainnet_sns_neurons, validate_sns_neuron_rows,
};
pub(in crate::sns::report) use proposals::{
    validate_mainnet_sns_proposal, validate_mainnet_sns_proposal_page,
    validate_mainnet_sns_proposals, validate_sns_proposal_rows,
};
pub use reward::MainnetSnsRewardNeuronPage;
pub(in crate::sns::report) use reward::{
    SnsRewardCollectionState, validate_mainnet_sns_reward_neuron_page,
};
pub(in crate::sns::report) use swap::{
    SNS_SWAP_QUERY_COUNT, canonicalize_mainnet_sns_swap, sns_swap_component_method,
};
pub use token::MainnetSnsToken;
pub use upgrade::MainnetSnsUpgrade;
pub(in crate::sns::report) use upgrade::{
    SNS_UPGRADE_QUERY_COUNT, canonicalize_mainnet_sns_upgrade,
};
