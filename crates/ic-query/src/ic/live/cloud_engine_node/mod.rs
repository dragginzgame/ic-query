//! Module: ic::live::cloud_engine_node
//!
//! Responsibility: official Dashboard CloudEngine Type4 node URL and wire handling.
//! Does not own: source validation, report projection, native queries, rendering, or caching.
//! Boundary: performs one explicitly filtered list request or one exact node request.

use super::{
    LiveIcSource, dashboard_resource_url,
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
    fn fetch_cloud_engine_node_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        node_provider_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<CloudEngineNodeListSourceData, IcHostError>> {
        Box::pin(async move {
            let node_provider_id = node_provider_id
                .map(|value| canonical_request_principal("node_provider_id", value))
                .transpose()?;
            let url = cloud_engine_node_list_url(&request.endpoint, node_provider_id.as_deref())?;
            let wire: DashboardNodes = self.fetch_live(url).await?;
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
        })
    }

    fn fetch_cloud_engine_node_info<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        node_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<CloudEngineNodeInfoSourceData, IcHostError>> {
        Box::pin(async move {
            let node_id = canonical_request_principal("node_id", node_id)?;
            let url = cloud_engine_node_info_url(&request.endpoint, &node_id)?;
            let node: DashboardNode = self.fetch_live(url).await?;
            Ok(CloudEngineNodeInfoSourceData {
                source: request.clone(),
                node_id,
                node: node.into_public(),
            })
        })
    }
}

fn cloud_engine_node_list_url(
    endpoint: &str,
    node_provider_id: Option<&str>,
) -> Result<Url, IcHostError> {
    let mut url = dashboard_resource_url(endpoint, &["nodes"])?;
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
    dashboard_resource_url(endpoint, &["nodes", node_id])
}

fn included_statuses() -> Vec<String> {
    CLOUD_ENGINE_NODE_INCLUDED_STATUSES
        .into_iter()
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests;
