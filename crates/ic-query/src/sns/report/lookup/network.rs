//! Module: sns::report::lookup::network
//!
//! Responsibility: SNS report network guard.
//! Does not own: command parsing, live transport, or report construction.
//! Boundary: rejects unsupported networks before mainnet-only SNS queries.

use crate::sns::report::SnsHostError;
use crate::subnet_catalog::MAINNET_NETWORK;

/// Ensure an SNS report request targets the supported mainnet network.
pub(in crate::sns::report) fn enforce_mainnet_network(network: &str) -> Result<(), SnsHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(SnsHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}
