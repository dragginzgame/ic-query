use super::*;

const PROVIDER_ID: &str = "rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae";

#[test]
fn provider_urls_select_complete_and_exact_resources() {
    assert_eq!(
        provider_list_url("https://ic-api.internetcomputer.org/api/v3/")
            .expect("provider list URL")
            .as_str(),
        "https://ic-api.internetcomputer.org/api/v3/node-providers"
    );
    assert_eq!(
        provider_info_url("https://ic-api.internetcomputer.org/api/v3", PROVIDER_ID)
            .expect("provider info URL")
            .as_str(),
        format!("https://ic-api.internetcomputer.org/api/v3/node-providers/{PROVIDER_ID}")
    );
}

#[test]
fn provider_wire_preserves_cloud_engine_and_general_fields() {
    let wire: DashboardNodeProviderList = serde_json::from_str(&format!(
            r#"{{"node_providers":[{{
                "principal_id":"{PROVIDER_ID}",
                "display_name":"Provider",
                "website":"example.com",
                "logo_url":null,
                "location_count":1,
                "locations":[{{"dc_key":"dc1","display_name":"One","latitude":1.5,"longitude":2.5,"owner":"Owner","region":"Region"}}],
                "cloud_engine_location_count":1,
                "cloud_engine_locations":[{{"dc_key":"dc1","display_name":"One","latitude":1.5,"longitude":2.5,"owner":"Owner","region":"Region"}}],
                "total_cloud_engine_nodes":5,
                "total_cloud_engine_unassigned_nodes":4,
                "total_cloud_engines":1,
                "total_node_allowance":7,
                "total_nodes":8,
                "total_rewardable_nodes":6,
                "total_subnets":2,
                "total_unassigned_nodes":3,
                "additive":"ignored"
            }}]}}"#
        ))
        .expect("provider wire");

    let provider = &wire.node_providers[0];
    assert_eq!(provider.total_cloud_engine_nodes, 5);
    assert_eq!(provider.total_nodes, 8);
    assert_eq!(provider.cloud_engine_locations[0].dc_key, "dc1");
}
