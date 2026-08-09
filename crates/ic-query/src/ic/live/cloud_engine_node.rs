//! Module: ic::live::cloud_engine_node
//!
//! Responsibility: official Dashboard CloudEngine Type4 node URL and wire handling.
//! Does not own: source validation, report projection, native queries, rendering, or caching.
//! Boundary: performs one explicitly filtered list request or one exact node request.

use super::{
    LiveIcSource, append_path_segments, dashboard_base_url, fetch_live,
    node_status::{DashboardNode, DashboardNodes},
};
use crate::{
    cloud_engine::{
        CLOUD_ENGINE_NODE_INCLUDED_STATUSES, CLOUD_ENGINE_NODE_REWARD_TYPE,
        CloudEngineNodeInfoSourceData, CloudEngineNodeListSourceData, CloudEngineNodeSource,
    },
    ic::{IcHostError, IcSourceRequest, canonical_request_principal},
};
use url::Url;

impl CloudEngineNodeSource for LiveIcSource {
    fn fetch_cloud_engine_node_list(
        &self,
        request: &IcSourceRequest,
        node_provider_id: Option<&str>,
    ) -> Result<CloudEngineNodeListSourceData, IcHostError> {
        let node_provider_id = node_provider_id
            .map(|value| canonical_request_principal("node_provider_id", value))
            .transpose()?;
        let url = cloud_engine_node_list_url(&request.endpoint, node_provider_id.as_deref())?;
        let wire: DashboardNodes = fetch_live(url)?;
        Ok(CloudEngineNodeListSourceData {
            source: request.clone(),
            requested_node_provider_id: node_provider_id,
            node_reward_type: CLOUD_ENGINE_NODE_REWARD_TYPE.to_string(),
            included_statuses: included_statuses(),
            nodes: wire
                .nodes
                .into_iter()
                .map(DashboardNode::into_public)
                .collect(),
        })
    }

    fn fetch_cloud_engine_node_info(
        &self,
        request: &IcSourceRequest,
        node_id: &str,
    ) -> Result<CloudEngineNodeInfoSourceData, IcHostError> {
        let node_id = canonical_request_principal("node_id", node_id)?;
        let url = cloud_engine_node_info_url(&request.endpoint, &node_id)?;
        let node: DashboardNode = fetch_live(url)?;
        Ok(CloudEngineNodeInfoSourceData {
            source: request.clone(),
            node_id,
            node: node.into_public(),
        })
    }
}

fn cloud_engine_node_list_url(
    endpoint: &str,
    node_provider_id: Option<&str>,
) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["nodes"])?;
    {
        let mut query = url.query_pairs_mut();
        for status in CLOUD_ENGINE_NODE_INCLUDED_STATUSES {
            query.append_pair("include_status", status);
        }
        query.append_pair("include_node_reward_type", CLOUD_ENGINE_NODE_REWARD_TYPE);
        query.append_pair("sort_by", "node_id");
        if let Some(node_provider_id) = node_provider_id {
            query.append_pair("node_provider_id", node_provider_id);
        }
    }
    Ok(url)
}

fn cloud_engine_node_info_url(endpoint: &str, node_id: &str) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["nodes", node_id])?;
    Ok(url)
}

fn included_statuses() -> Vec<String> {
    CLOUD_ENGINE_NODE_INCLUDED_STATUSES
        .into_iter()
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
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
}
