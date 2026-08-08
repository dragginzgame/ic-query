//! Module: ic::live::node_status
//!
//! Responsibility: live official Dashboard node-resource URL and wire conversion.
//! Does not own: snapshot validation, cache policy, projections, or rendering.
//! Boundary: performs one finite read-only `/nodes` request with no follow-up calls.

use super::{LiveIcSource, append_path_segments, dashboard_base_url, fetch_live};
use crate::ic::{
    IcHostError, IcNodeStatusRow, IcNodeStatusScope, IcNodeStatusSource, IcNodeStatusSourceData,
    IcSourceRequest,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcNodeStatusSource for LiveIcSource {
    fn fetch_node_status_snapshot(
        &self,
        request: &IcSourceRequest,
    ) -> Result<IcNodeStatusSourceData, IcHostError> {
        let url = node_status_url(&request.endpoint)?;
        let wire: DashboardNodes = fetch_live(url)?;
        Ok(IcNodeStatusSourceData {
            source: request.clone(),
            scope: IcNodeStatusScope::DashboardMainnetDefault,
            cloud_engine_nodes_included: false,
            nodes: wire
                .nodes
                .into_iter()
                .map(DashboardNode::into_public)
                .collect(),
        })
    }
}

fn node_status_url(endpoint: &str) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["nodes"])?;
    Ok(url)
}

///
/// DashboardNodes
///
/// Shared wire envelope for official Dashboard node collections.
///

#[derive(SerdeDeserialize)]
pub(super) struct DashboardNodes {
    /// Raw Dashboard node rows.
    pub(super) nodes: Vec<DashboardNode>,
}

///
/// DashboardNode
///
/// Shared wire row for default-mainnet, explicitly filtered, and exact node resources.
///

#[derive(SerdeDeserialize)]
pub(super) struct DashboardNode {
    alertname: String,
    cloud_engine_subnet_id: Option<String>,
    dc_id: String,
    dc_name: String,
    guestos_tee_active: Option<bool>,
    guestos_version: Option<String>,
    ip_address: Option<String>,
    ipv4_connectivity_status: Option<bool>,
    node_hardware_generation: Option<String>,
    node_id: String,
    node_operator_id: String,
    node_provider_id: String,
    node_provider_name: String,
    node_reward_type: String,
    node_type: String,
    owner: String,
    region: String,
    status: String,
    subnet_id: Option<String>,
}

impl DashboardNode {
    /// Convert the shared wire row without changing raw Dashboard evidence.
    pub(super) fn into_public(self) -> IcNodeStatusRow {
        IcNodeStatusRow {
            node_id: self.node_id,
            node_operator_id: self.node_operator_id,
            node_provider_id: self.node_provider_id,
            node_provider_name: self.node_provider_name,
            node_type: self.node_type,
            node_reward_type: self.node_reward_type,
            status: self.status,
            alert_name: (!self.alertname.is_empty()).then_some(self.alertname),
            subnet_id: self.subnet_id,
            cloud_engine_subnet_id: self.cloud_engine_subnet_id,
            data_center_id: self.dc_id,
            data_center_name: self.dc_name,
            owner: self.owner,
            region: self.region,
            guestos_version: self.guestos_version,
            guestos_tee_active: self.guestos_tee_active,
            ip_address: self.ip_address,
            ipv4_connectivity_status: self.ipv4_connectivity_status,
            node_hardware_generation: self.node_hardware_generation,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
