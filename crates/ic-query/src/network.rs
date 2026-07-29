//! Module: network
//!
//! Responsibility: enforce shared built-in network support policy.
//! Does not own: command parsing, endpoint selection, or family-specific errors.
//! Boundary: maps an unsupported network identity into the caller's typed error.

use crate::subnet_catalog::MAINNET_NETWORK;

pub fn enforce_mainnet_network_with<Error>(
    network: &str,
    unsupported_network: impl FnOnce(String) -> Error,
) -> Result<(), Error> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(unsupported_network(network.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainnet_policy_preserves_the_rejected_network() {
        assert_eq!(
            enforce_mainnet_network_with(MAINNET_NETWORK, |network| network),
            Ok(())
        );
        assert_eq!(
            enforce_mainnet_network_with("local", |network| network),
            Err("local".to_string())
        );
    }
}
