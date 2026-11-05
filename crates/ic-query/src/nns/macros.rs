//! Code generation for structurally identical NNS leaf report mechanics.

macro_rules! impl_nns_mainnet_network_enforcer {
    ($error:ident, $collection:expr) => {
        fn enforce_mainnet_network(network: &str) -> Result<(), $error> {
            crate::network::enforce_mainnet_network_with(network, |network| {
                $error::UnsupportedNetwork {
                    collection: $collection,
                    network,
                }
            })
        }
    };
    ($error:ident) => {
        fn enforce_mainnet_network(network: &str) -> Result<(), $error> {
            crate::network::enforce_mainnet_network_with(network, |network| {
                $error::UnsupportedNetwork { network }
            })
        }
    };
}
