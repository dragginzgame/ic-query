//! Module: sns::report::live::fetch
//!
//! Responsibility: group live SNS fetch helpers.
//! Does not own: Candid wire definitions, report assembly, cache IO, or rendering.
//! Boundary: wraps async IC queries behind synchronous report-source helpers.

mod canisters;
mod list;
mod metrics;
mod neurons;
mod params;
mod proposals;
mod reward;
mod swap;
mod token;
mod upgrade;

use super::{
    super::{MainnetSns, SnsHostError},
    query::principal_from_text,
};
use ic_platform::candid::contract::Principal;

pub(super) use canisters::fetch_mainnet_sns_canisters;
pub(super) use list::{
    fetch_mainnet_sns_inventory, fetch_mainnet_sns_lifecycles, fetch_mainnet_sns_metadata,
};
pub(super) use metrics::fetch_mainnet_sns_metrics;
pub(super) use neurons::{
    fetch_mainnet_sns_neuron, fetch_mainnet_sns_neuron_page, fetch_mainnet_sns_neurons,
};
pub(super) use params::fetch_mainnet_sns_params;
pub(super) use proposals::{
    fetch_mainnet_sns_proposal, fetch_mainnet_sns_proposal_page, fetch_mainnet_sns_proposals,
};
pub(super) use reward::{fetch_mainnet_sns_reward_event, fetch_mainnet_sns_reward_neuron_page};
pub(super) use swap::fetch_mainnet_sns_swap;
pub(super) use token::fetch_mainnet_sns_token;
pub(super) use upgrade::{fetch_mainnet_sns_running_version, fetch_mainnet_sns_upgrade};

/// Parse the governance canister id for one resolved SNS.
fn governance_canister(sns: &MainnetSns) -> Result<Principal, SnsHostError> {
    principal_from_text(
        &sns.canisters.governance_canister_id,
        "governance_canister_id",
    )
}
