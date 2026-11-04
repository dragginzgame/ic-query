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
