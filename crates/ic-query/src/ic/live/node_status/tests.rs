use super::*;

#[test]
fn node_status_url_appends_one_unfiltered_resource() {
    for endpoint in [
        "https://ic-api.internetcomputer.org/api/v3",
        "https://ic-api.internetcomputer.org/api/v3/",
    ] {
        let url = node_status_url(endpoint).expect("node-status URL");
        assert_eq!(
            url.as_str(),
            "https://ic-api.internetcomputer.org/api/v3/nodes"
        );
        assert!(url.query().is_none());
    }
}

#[test]
fn node_wire_preserves_status_evidence_and_ignores_additive_fields() {
    let wire: DashboardNodes = serde_json::from_str(
        r#"{
                "nodes": [{
                    "alertname": "IC_Node_HostVersionBehind",
                    "cloud_engine_subnet_id": null,
                    "dc_id": "fm1",
                    "dc_name": "Fremont",
                    "guestos_tee_active": true,
                    "guestos_version": "abc",
                    "ip_address": "2001:db8::1",
                    "ipv4_connectivity_status": true,
                    "node_hardware_generation": null,
                    "node_id": "aaaaa-aa",
                    "node_operator_id": "2vxsx-fae",
                    "node_provider_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
                    "node_provider_name": "Provider",
                    "node_reward_type": "Type3dot1",
                    "node_type": "REPLICA",
                    "owner": "Owner",
                    "region": "North America,US,California",
                    "status": "DEGRADED",
                    "subnet_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
                    "future_field": {"value": 1}
                }],
                "future_top_level": true
            }"#,
    )
    .expect("current node payload");

    let row = wire
        .nodes
        .into_iter()
        .next()
        .expect("one row")
        .into_public();
    assert_eq!(row.status, "DEGRADED");
    assert_eq!(row.alert_name.as_deref(), Some("IC_Node_HostVersionBehind"));
    assert_eq!(row.node_hardware_generation, None);
}
