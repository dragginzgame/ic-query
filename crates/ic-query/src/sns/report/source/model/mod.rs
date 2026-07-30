//! Module: sns::report::source::model
//!
//! Responsibility: group SNS source result models.
//! Does not own: live transport, report DTOs, cache IO, or rendering.
//! Boundary: re-exports source-layer data passed from fetchers to builders.

mod canisters;
mod fetch;
mod list;
mod neurons;
mod proposals;
mod token;

pub use canisters::MainnetSnsCanisterInventory;
pub(in crate::sns::report) use canisters::{
    SNS_CANISTER_HEALTH_CALL_TYPE, SNS_CANISTER_HEALTH_METHOD, SNS_CANISTER_INVENTORY_METHOD,
    canonicalize_mainnet_sns_canister_inventory,
};
pub use fetch::SnsSourceRequest;
pub use list::{MainnetSns, MainnetSnsList};
pub(in crate::sns::report) use list::{MainnetSnsCanisters, validate_mainnet_sns_list};
pub use neurons::{MainnetSnsNeuronPage, MainnetSnsNeurons, SnsNeuronId};
pub use proposals::{MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals};
pub use token::MainnetSnsToken;
