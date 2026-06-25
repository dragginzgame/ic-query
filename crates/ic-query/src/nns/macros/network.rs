macro_rules! impl_nns_mainnet_network_enforcer {
    ($error:ident) => {
        fn enforce_mainnet_network(network: &str) -> Result<(), $error> {
            if network == crate::subnet_catalog::MAINNET_NETWORK {
                return Ok(());
            }
            Err($error::UnsupportedNetwork {
                network: network.to_string(),
            })
        }
    };
}
