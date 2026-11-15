//! Public cloud-engine evidence for Canic cost and placement decisions.
//!
//! Marketplace declarations do not establish available capacity or binding quotes.

use crate::{ReadError, mainnet};
use ic_platform::agent::Agent;

pub mod available;
pub mod candidates;
pub mod engine;
pub mod prices;

const CANISTER: &str = "q6cfj-fyaaa-aaaar-qb77q-cai";

struct Observation {
    bytes: Vec<u8>,
    observed_at: u64,
}

// Shared transport for the two concrete cloud-engine reports, not a public
// arbitrary-method API or a generic pagination framework.
async fn query(agent: &Agent, method: &str, argument: Vec<u8>) -> Result<Observation, ReadError> {
    let canister = CANISTER.parse().expect("fixed mainnet engine principal");
    let bytes = mainnet::read(agent.query(&canister, method).with_arg(argument).call()).await?;
    Ok(Observation {
        bytes,
        observed_at: mainnet::observed_at()?,
    })
}
