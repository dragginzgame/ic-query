use super::*;

#[test]
fn json_preserves_full_width_raw_values_and_zero_observations() {
    for (value, cycles) in [(0, 0), (u64::MAX, u128::MAX)] {
        let report = SubnetMetricsReport {
            schema: 1,
            subnet_id: Principal::from_slice(&[1]),
            num_canisters: value,
            canister_state_bytes: value,
            consumed_cycles_total: cycles,
            update_transactions_total: value,
            evidence: SubnetMetricsEvidence {
                endpoint: "fixture",
                source_path: "/read_state".into(),
                state_tree_path: "/subnet/fixture/metrics".into(),
                observed_at_unix_seconds: 123,
                state_certificate_verified: false,
            },
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["num_canisters"], value);
        assert_eq!(json["canister_state_bytes"], value);
        assert_eq!(json["update_transactions_total"], value);
        assert_eq!(json["consumed_cycles_total"], cycles.to_string());
        assert_eq!(json["evidence"]["state_certificate_verified"], false);
        assert_eq!(json["evidence"]["observed_at_unix_seconds"], 123);
        assert!(json.get("uptime").is_none());
        assert!(json.get("available_capacity").is_none());
    }
}
