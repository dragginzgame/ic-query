//! Module: ic::live::node_status
//!
//! Responsibility: live official Dashboard node-resource URL and wire conversion.
//! Does not own: snapshot validation, cache policy, projections, or rendering.
//! Boundary: performs one finite read-only `/nodes` request with no follow-up calls.

use super::{LiveIcSource, dashboard_resource_url};
use crate::ic::{
    IcHostError, IcNodeStatusRow, IcNodeStatusScope, IcNodeStatusSource, IcNodeStatusSourceData,
    IcSourceRequest,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcNodeStatusSource for LiveIcSource {
    fn fetch_node_status_snapshot<'a>(
        &'a self,
        request: &'a IcSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<IcNodeStatusSourceData, IcHostError>> {
        Box::pin(async move {
            let url = node_status_url(&request.endpoint)?;
            let wire: DashboardNodes = self.fetch_live(url).await?;
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
        })
    }
}

fn node_status_url(endpoint: &str) -> Result<Url, IcHostError> {
    dashboard_resource_url(endpoint, &["nodes"])
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
mod tests;
