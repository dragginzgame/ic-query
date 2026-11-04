use crate::cloud_engine::cloud_engine_node_info_report_text;
use crate::cloud_engine::cloud_engine_node_list_report_text;
use ic_query::cloud_engine::CLOUD_ENGINE_NODE_INCLUDED_STATUSES;
use ic_query::cloud_engine::CLOUD_ENGINE_NODE_REWARD_TYPE;
use ic_query::cloud_engine::CloudEngineNodeInfoReport;
use ic_query::cloud_engine::CloudEngineNodeListReport;
use ic_query::cloud_engine::CloudEngineNodeRow;
use ic_query::ic::IcDashboardReportProvenance;
use ic_query::ic::IcNodeStatusCounts;

const NODE_A: &str = "53amq-7hjxu-6lxaj-o2sp6-kmngy-qa22h-b7bo6-oeyyn-fkqnv-7tauf-7qe";
const PROVIDER: &str = "bvcsg-3od6r-jnydw-eysln-aql7w-td5zn-ay5m6-sibd2-jzojt-anwag-mqe";
const OPERATOR: &str = "e3aue-mkha2-6zddy-xbmd7-3oybi-3nfoh-3bwgn-izbjn-uuqx2-ykc2z-7qe";
const CLOUD_ENGINE_SUBNET: &str = "nx5oj-b2azr-x3alh-sgf7i-duhfw-bflus-hisa2-5n2oq-tv7sd-haspd-cae";

#[test]
fn text_separates_dashboard_provenance_from_type4_nodes() {
    let node = node(NODE_A, "UP");
    let list = CloudEngineNodeListReport {
        provenance: provenance(),
        node_reward_type: CLOUD_ENGINE_NODE_REWARD_TYPE.to_string(),
        included_statuses: included_statuses(),
        requested_node_provider_id: None,
        status_counts: IcNodeStatusCounts {
            total: 1,
            up: 1,
            ..IcNodeStatusCounts::default()
        },
        node_provider_count: 1,
        cloud_engine_subnet_count: 1,
        unassigned_cloud_engine_node_count: 0,
        nodes: vec![node.clone()],
    };
    let info = CloudEngineNodeInfoReport {
        provenance: provenance(),
        node,
    };

    let list_text = cloud_engine_node_list_report_text(&list);
    let info_text = cloud_engine_node_info_report_text(&info);
    assert!(list_text.contains("node_count: 1\n"));
    assert!(list_text.contains("unassigned_cloud_engine_node_count: 0\n\nCloudEngine nodes\n"));
    assert!(info_text.contains("node_reward_type: Type4"));
    assert!(info_text.contains("cloud_engine_subnet_id: nx5oj"));
}

fn provenance() -> IcDashboardReportProvenance {
    IcDashboardReportProvenance {
        schema_version: 1,
        network: "ic".to_string(),
        authority: "official_ic_dashboard_api".to_string(),
        source_endpoint: "https://ic-api.internetcomputer.org/api/v3".to_string(),
        fetched_at: "2026-08-08T12:00:00Z".to_string(),
        fetched_by: "test".to_string(),
        certified: false,
        point_in_time_guaranteed: false,
    }
}

fn included_statuses() -> Vec<String> {
    CLOUD_ENGINE_NODE_INCLUDED_STATUSES
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn node(node_id: &str, status: &str) -> CloudEngineNodeRow {
    CloudEngineNodeRow {
        node_id: node_id.to_string(),
        node_operator_id: OPERATOR.to_string(),
        node_provider_id: PROVIDER.to_string(),
        node_provider_name: "DFINITY Stiftung".to_string(),
        node_type: "UNASSIGNED".to_string(),
        node_reward_type: CLOUD_ENGINE_NODE_REWARD_TYPE.to_string(),
        status: status.to_string(),
        alert_name: None,
        subnet_id: None,
        cloud_engine_subnet_id: Some(CLOUD_ENGINE_SUBNET.to_string()),
        data_center_id: "tp1".to_string(),
        data_center_name: "Tampa".to_string(),
        owner: "Flexential".to_string(),
        region: "North America,US,Florida".to_string(),
        guestos_version: Some("release".to_string()),
        guestos_tee_active: Some(false),
        ip_address: Some("2001:db8::1".to_string()),
        ipv4_connectivity_status: Some(false),
        node_hardware_generation: Some("Gen1".to_string()),
    }
}
