use super::super::SnsHostError;
use crate::subnet_catalog::MAINNET_NETWORK;

pub(in crate::sns::report) fn enforce_mainnet_network(network: &str) -> Result<(), SnsHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(SnsHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}
