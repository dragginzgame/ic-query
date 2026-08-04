use super::{FixtureSnsDiscoverySource, GOVERNANCE_A};
use crate::sns::report::{source::*, tests::*};

///
/// FixtureSnsUpgradeSource
///
/// Successful bounded SNS upgrade source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsUpgradeSource;

delegate_sns_discovery!(FixtureSnsUpgradeSource);

impl SnsUpgradeSource for FixtureSnsUpgradeSource {
    fn fetch_sns_upgrade(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsUpgrade, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        Ok(fixture_mainnet_sns_upgrade())
    }
}

///
/// MutatingFixtureSnsUpgradeSource
///
/// SNS upgrade source that mutates otherwise valid evidence for invariant tests.
///

pub(in crate::sns::report::tests) struct MutatingFixtureSnsUpgradeSource(
    pub(in crate::sns::report::tests) fn(&mut MainnetSnsUpgrade),
);

delegate_sns_discovery!(MutatingFixtureSnsUpgradeSource);

impl SnsUpgradeSource for MutatingFixtureSnsUpgradeSource {
    fn fetch_sns_upgrade(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<MainnetSnsUpgrade, SnsHostError> {
        let mut upgrade = fixture_mainnet_sns_upgrade();
        self.0(&mut upgrade);
        Ok(upgrade)
    }
}

pub(in crate::sns::report::tests) fn fixture_sns_version(seed: u8) -> SnsVersion {
    SnsVersion {
        archive_wasm_hash_hex: format!("{seed:02x}"),
        root_wasm_hash_hex: format!("{:02x}", seed + 1),
        swap_wasm_hash_hex: format!("{:02x}", seed + 2),
        ledger_wasm_hash_hex: format!("{:02x}", seed + 3),
        governance_wasm_hash_hex: format!("{:02x}", seed + 4),
        index_wasm_hash_hex: format!("{:02x}", seed + 5),
    }
}

fn fixture_mainnet_sns_upgrade() -> MainnetSnsUpgrade {
    MainnetSnsUpgrade {
        governance_canister_id: GOVERNANCE_A.to_string(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        running_version_method: SnsCanisterMethod::GetRunningSnsVersion,
        next_version_method: SnsCanisterMethod::GetNextSnsVersion,
        point_in_time_guaranteed: false,
        deployed_version: fixture_sns_version(1),
        pending_upgrade: Some(SnsPendingUpgrade {
            mark_failed_at_seconds: 1_780_617_600,
            checking_upgrade_lock: 9,
            proposal_id: 42,
            target_version: Some(fixture_sns_version(11)),
        }),
        next_version: Some(fixture_sns_version(21)),
        next_version_gap: None,
    }
}
