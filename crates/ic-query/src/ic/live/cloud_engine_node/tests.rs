use super::*;

const NODE_ID: &str = "53amq-7hjxu-6lxaj-o2sp6-kmngy-qa22h-b7bo6-oeyyn-fkqnv-7tauf-7qe";
const PROVIDER_ID: &str = "bvcsg-3od6r-jnydw-eysln-aql7w-td5zn-ay5m6-sibd2-jzojt-anwag-mqe";

#[test]
fn list_url_selects_type4_all_statuses_and_optional_provider() {
    let url = cloud_engine_node_list_url(
        "https://ic-api.internetcomputer.org/api/v3/",
        Some(PROVIDER_ID),
    )
    .expect("Type4 node URL");
    assert_eq!(url.path(), "/api/v3/nodes");
    let pairs = url.query_pairs().collect::<Vec<_>>();
    assert_eq!(
        pairs,
        vec![
            ("include_status".into(), "DOWN".into()),
            ("include_status".into(), "UP".into()),
            ("include_status".into(), "DISABLED".into()),
            ("include_status".into(), "DEGRADED".into()),
            ("include_node_reward_type".into(), "Type4".into()),
            ("sort_by".into(), "node_id".into()),
            ("node_provider_id".into(), PROVIDER_ID.into()),
        ]
    );
}

#[test]
fn exact_url_selects_one_node_resource() {
    assert_eq!(
        cloud_engine_node_info_url("https://ic-api.internetcomputer.org/api/v3", NODE_ID)
            .expect("exact node URL")
            .as_str(),
        format!("https://ic-api.internetcomputer.org/api/v3/nodes/{NODE_ID}")
    );
}

#[test]
fn shared_node_wire_preserves_type4_specific_fields() {
    let node: DashboardNode = serde_json::from_str(&format!(
            r#"{{
                "alertname":"",
                "cloud_engine_subnet_id":"nx5oj-b2azr-x3alh-sgf7i-duhfw-bflus-hisa2-5n2oq-tv7sd-haspd-cae",
                "dc_id":"tp1",
                "dc_name":"Tampa",
                "guestos_tee_active":false,
                "guestos_version":"release",
                "ip_address":"2001:db8::1",
                "ipv4_connectivity_status":false,
                "node_hardware_generation":"Gen1",
                "node_id":"{NODE_ID}",
                "node_operator_id":"e3aue-mkha2-6zddy-xbmd7-3oybi-3nfoh-3bwgn-izbjn-uuqx2-ykc2z-7qe",
                "node_provider_id":"{PROVIDER_ID}",
                "node_provider_name":"DFINITY Stiftung",
                "node_reward_type":"Type4",
                "node_type":"UNASSIGNED",
                "owner":"Flexential",
                "region":"North America,US,Florida",
                "status":"UP",
                "subnet_id":null,
                "additive":true
            }}"#
        ))
        .expect("Type4 node wire");
    let row = node.into_public();
    assert_eq!(row.node_reward_type, "Type4");
    assert!(row.cloud_engine_subnet_id.is_some());
    assert_eq!(row.subnet_id, None);
}
