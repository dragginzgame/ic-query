//! Module: cloud_engine::node::host
//!
//! Responsibility: validate and build official Dashboard CloudEngine Type4 node reports.
//! Does not own: HTTP transport, native control-plane queries, rendering, or caching.
//! Boundary: validates the explicit Type4/status/provider query echo before projection.

use super::{
    CLOUD_ENGINE_NODE_INCLUDED_STATUSES, CLOUD_ENGINE_NODE_REWARD_TYPE, CloudEngineNodeInfoReport,
    CloudEngineNodeInfoRequest, CloudEngineNodeInfoSourceData, CloudEngineNodeListReport,
    CloudEngineNodeListRequest, CloudEngineNodeListSourceData, CloudEngineNodeRow,
    MAX_CLOUD_ENGINE_NODE_ROWS,
};
use crate::ic::{
    IcHostError, IcSourceRequest, LiveIcSource, canonical_request_principal,
    canonicalize_node_status_rows_with_policy, dashboard_source_request, invalid_source,
    invalid_source_value, node_status_counts, report_provenance, validate_dashboard_network,
    validate_provenance,
};
use std::collections::HashSet;

///
/// CloudEngineNodeSource
///
/// Official Dashboard capability for complete Type4 and exact node observations.
///

pub trait CloudEngineNodeSource {
    /// Fetch the complete explicitly scoped Type4 node resource once.
    fn fetch_cloud_engine_node_list(
        &self,
        request: &IcSourceRequest,
        node_provider_id: Option<&str>,
    ) -> Result<CloudEngineNodeListSourceData, IcHostError>;

    /// Fetch one exact node observation once.
    fn fetch_cloud_engine_node_info(
        &self,
        request: &IcSourceRequest,
        node_id: &str,
    ) -> Result<CloudEngineNodeInfoSourceData, IcHostError>;
}

/// Build one live complete CloudEngine Type4 node report from the official Dashboard.
pub fn build_cloud_engine_node_list_report(
    request: &CloudEngineNodeListRequest,
) -> Result<CloudEngineNodeListReport, IcHostError> {
    build_cloud_engine_node_list_report_with_source(request, &LiveIcSource)
}

/// Build one complete CloudEngine Type4 node report through a custom Dashboard source.
pub fn build_cloud_engine_node_list_report_with_source(
    request: &CloudEngineNodeListRequest,
    source: &dyn CloudEngineNodeSource,
) -> Result<CloudEngineNodeListReport, IcHostError> {
    validate_dashboard_network(&request.network)?;
    let requested_node_provider_id = request
        .node_provider_id
        .as_deref()
        .map(|value| canonical_request_principal("node_provider_id", value))
        .transpose()?;
    let expected_source = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let mut source_data = source
        .fetch_cloud_engine_node_list(&expected_source, requested_node_provider_id.as_deref())?;
    validate_provenance(&expected_source, &source_data.source)?;
    if source_data.requested_node_provider_id != requested_node_provider_id {
        return invalid_source(format!(
            "node-provider filter is {:?}, expected {:?}",
            source_data.requested_node_provider_id, requested_node_provider_id
        ));
    }
    if source_data.node_reward_type != CLOUD_ENGINE_NODE_REWARD_TYPE {
        return invalid_source(format!(
            "node reward-type scope is {:?}, expected {CLOUD_ENGINE_NODE_REWARD_TYPE:?}",
            source_data.node_reward_type
        ));
    }
    let expected_statuses = included_statuses();
    if source_data.included_statuses != expected_statuses {
        return invalid_source(format!(
            "included status scope is {:?}, expected {:?}",
            source_data.included_statuses, expected_statuses
        ));
    }
    canonicalize_node_status_rows_with_policy(
        &mut source_data.nodes,
        MAX_CLOUD_ENGINE_NODE_ROWS,
        false,
    )
    .map_err(invalid_source_value)?;
    validate_type4_nodes(
        &source_data.nodes,
        requested_node_provider_id.as_deref(),
        true,
    )?;

    let node_providers = source_data
        .nodes
        .iter()
        .map(|node| node.node_provider_id.as_str())
        .collect::<HashSet<_>>();
    let cloud_engine_subnets = source_data
        .nodes
        .iter()
        .filter_map(|node| node.cloud_engine_subnet_id.as_deref())
        .collect::<HashSet<_>>();
    let unassigned_cloud_engine_node_count = source_data
        .nodes
        .iter()
        .filter(|node| node.cloud_engine_subnet_id.is_none())
        .count();

    Ok(CloudEngineNodeListReport {
        provenance: report_provenance(source_data.source),
        node_reward_type: source_data.node_reward_type,
        included_statuses: source_data.included_statuses,
        requested_node_provider_id,
        node_count: source_data.nodes.len(),
        status_counts: node_status_counts(source_data.nodes.iter()),
        node_provider_count: node_providers.len(),
        cloud_engine_subnet_count: cloud_engine_subnets.len(),
        unassigned_cloud_engine_node_count,
        nodes: source_data.nodes,
    })
}

/// Build one live exact CloudEngine Type4 node report from the official Dashboard.
pub fn build_cloud_engine_node_info_report(
    request: &CloudEngineNodeInfoRequest,
) -> Result<CloudEngineNodeInfoReport, IcHostError> {
    build_cloud_engine_node_info_report_with_source(request, &LiveIcSource)
}

/// Build one exact CloudEngine Type4 node report through a custom Dashboard source.
pub fn build_cloud_engine_node_info_report_with_source(
    request: &CloudEngineNodeInfoRequest,
    source: &dyn CloudEngineNodeSource,
) -> Result<CloudEngineNodeInfoReport, IcHostError> {
    validate_dashboard_network(&request.network)?;
    let requested_node_id = canonical_request_principal("node_id", &request.node_id)?;
    let expected_source = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_cloud_engine_node_info(&expected_source, &requested_node_id)?;
    validate_provenance(&expected_source, &source_data.source)?;
    if source_data.node_id != requested_node_id {
        return invalid_source(format!(
            "node target is {:?}, expected {requested_node_id:?}",
            source_data.node_id
        ));
    }
    let mut nodes = vec![source_data.node];
    canonicalize_node_status_rows_with_policy(&mut nodes, 1, true).map_err(invalid_source_value)?;
    validate_type4_nodes(&nodes, None, false)?;
    let node = nodes
        .into_iter()
        .next()
        .ok_or_else(|| invalid_source_value("exact node source returned no row"))?;
    if node.node_id != requested_node_id {
        return invalid_source(format!(
            "returned node id is {:?}, expected {requested_node_id:?}",
            node.node_id
        ));
    }

    Ok(CloudEngineNodeInfoReport {
        provenance: report_provenance(source_data.source),
        node,
    })
}

fn validate_type4_nodes(
    nodes: &[CloudEngineNodeRow],
    requested_node_provider_id: Option<&str>,
    require_included_status: bool,
) -> Result<(), IcHostError> {
    for node in nodes {
        if node.node_reward_type != CLOUD_ENGINE_NODE_REWARD_TYPE {
            return invalid_source(format!(
                "node {} reward type is {:?}, expected {CLOUD_ENGINE_NODE_REWARD_TYPE:?}",
                node.node_id, node.node_reward_type
            ));
        }
        if require_included_status
            && !CLOUD_ENGINE_NODE_INCLUDED_STATUSES.contains(&node.status.as_str())
        {
            return invalid_source(format!(
                "node {} status {:?} was not in the explicitly requested scope",
                node.node_id, node.status
            ));
        }
        if let Some(expected_provider) = requested_node_provider_id
            && node.node_provider_id != expected_provider
        {
            return invalid_source(format!(
                "node {} provider is {:?}, expected requested provider {expected_provider:?}",
                node.node_id, node.node_provider_id
            ));
        }
    }
    Ok(())
}

fn included_statuses() -> Vec<String> {
    CLOUD_ENGINE_NODE_INCLUDED_STATUSES
        .into_iter()
        .map(str::to_string)
        .collect()
}
