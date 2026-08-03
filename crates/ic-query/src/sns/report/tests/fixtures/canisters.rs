use super::sns::{FixtureSnsDiscoverySource, GOVERNANCE_A, INDEX_A, ROOT_A};
use crate::sns::report::tests::*;

///
/// FixtureSnsCanisterSource
///
/// Successful SNS Root inventory and health source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsCanisterSource;

delegate_sns_discovery!(FixtureSnsCanisterSource);

impl SnsCanisterSource for FixtureSnsCanisterSource {
    fn fetch_sns_canisters(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsCanisterInventory, SnsHostError> {
        assert_eq!(sns.root_canister_id, ROOT_A);
        Ok(MainnetSnsCanisterInventory {
            inventory_method: "list_sns_canisters".to_string(),
            health_method: "get_sns_canisters_summary".to_string(),
            health_call_type: SnsCanisterCallType::IngressUpdate,
            health_update_canister_list: false,
            point_in_time_guaranteed: false,
            canisters: vec![
                SnsCanisterRow {
                    role: SnsCanisterRole::Extension,
                    canister_id: INDEX_A.to_string(),
                    status: None,
                    module_hash_hex: None,
                    cycles: None,
                    memory_size: None,
                    idle_cycles_burned_per_day: None,
                    controllers: Vec::new(),
                },
                SnsCanisterRow {
                    role: SnsCanisterRole::Root,
                    canister_id: ROOT_A.to_string(),
                    status: Some(SnsCanisterStatus::Running),
                    module_hash_hex: Some("01020304".to_string()),
                    cycles: Some("1000000".to_string()),
                    memory_size: Some("2000000".to_string()),
                    idle_cycles_burned_per_day: Some("3000".to_string()),
                    controllers: vec![GOVERNANCE_A.to_string()],
                },
            ],
            gaps: vec![SnsCanisterGap {
                kind: SnsCanisterGapKind::HealthUnsupported,
                role: SnsCanisterRole::Extension,
                inventory_canister_id: Some(INDEX_A.to_string()),
                summary_canister_id: None,
            }],
        })
    }
}
