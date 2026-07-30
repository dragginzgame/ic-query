//! Module: sns::report::source
//!
//! Responsibility: group SNS source models and source traits.
//! Does not own: live transport implementations, cache IO, report assembly, or rendering.
//! Boundary: exposes source-layer contracts used by report builders and tests.

mod model;
mod traits;

pub(in crate::sns::report) use model::MainnetSnsCanisters;
pub use model::{
    MainnetSns, MainnetSnsCanisterInventory, MainnetSnsList, MainnetSnsNeuronPage,
    MainnetSnsNeurons, MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals,
    MainnetSnsToken, SnsNeuronId, SnsSourceRequest,
};
pub(in crate::sns::report) use model::{
    SNS_CANISTER_HEALTH_CALL_TYPE, SNS_CANISTER_HEALTH_METHOD, SNS_CANISTER_INVENTORY_METHOD,
    canonicalize_mainnet_sns_canister_inventory, validate_mainnet_sns_list,
};
pub use traits::{
    SnsCanisterSource, SnsListSource, SnsNeuronsSource, SnsParamsSource, SnsProposalSource,
    SnsProposalsSource, SnsTokenSource,
};
