//! Module: ic::source::node_status
//!
//! Responsibility: finite Dashboard node-status source contract and snapshot validation.
//! Does not own: HTTP transport, cache policy, aggregate projections, or rendering.
//! Boundary: converts one untrusted source response into a canonical observed snapshot.

use super::{
    canonical_request_principal, invalid_source, invalid_source_value, report_provenance,
    validate_provenance,
};
use crate::ic::{
    IcHostError, IcNodeStatusObservation, IcNodeStatusRow, IcNodeStatusScope, IcNodeStatusSnapshot,
    IcNodeStatusSourceData, IcSourceRequest, MAX_IC_NODE_STATUS_ROWS,
    node_status::node_status_group_counts,
};
use std::collections::{HashMap, HashSet};

///
/// IcNodeStatusSource
///
/// Source contract for one finite official Dashboard node-status snapshot.
///

pub trait IcNodeStatusSource {
    /// Fetch the Dashboard's default public-mainnet node resource in one request.
    fn fetch_node_status_snapshot(
        &self,
        request: &IcSourceRequest,
    ) -> Result<IcNodeStatusSourceData, IcHostError>;
}

pub(in crate::ic) fn node_status_snapshot_from_source(
    request: &IcSourceRequest,
    mut source: IcNodeStatusSourceData,
) -> Result<IcNodeStatusSnapshot, IcHostError> {
    validate_provenance(request, &source.source)?;
    if source.scope != IcNodeStatusScope::DashboardMainnetDefault {
        return invalid_source(format!(
            "node-status scope is {:?}, expected dashboard_mainnet_default",
            source.scope
        ));
    }
    if source.cloud_engine_nodes_included {
        return invalid_source(
            "default public-mainnet node scope cannot claim cloud-engine node inclusion",
        );
    }
    validate_node_status_rows(&mut source.nodes)?;
    validate_default_node_scope(&source.nodes)?;
    let counts = node_status_group_counts(source.nodes.iter());

    Ok(IcNodeStatusSnapshot {
        observation: IcNodeStatusObservation {
            source: report_provenance(source.source),
            scope: source.scope,
            cloud_engine_nodes_included: source.cloud_engine_nodes_included,
            cache: None,
        },
        node_count: source.nodes.len(),
        counts,
        nodes: source.nodes,
    })
}

pub(in crate::ic) fn validate_default_node_scope(
    nodes: &[IcNodeStatusRow],
) -> Result<(), IcHostError> {
    if let Some(node) = nodes
        .iter()
        .find(|node| node.cloud_engine_subnet_id.is_some())
    {
        return invalid_source(format!(
            "node {} contains cloud-engine Subnet evidence in the default public-mainnet scope",
            node.node_id
        ));
    }
    Ok(())
}

pub(in crate::ic) fn validate_node_status_rows(
    nodes: &mut [IcNodeStatusRow],
) -> Result<(), IcHostError> {
    if u32::try_from(nodes.len()).unwrap_or(u32::MAX) > MAX_IC_NODE_STATUS_ROWS {
        return invalid_source(format!(
            "source returned {} node rows; maximum is {MAX_IC_NODE_STATUS_ROWS}",
            nodes.len()
        ));
    }

    let mut seen = HashSet::with_capacity(nodes.len());
    let mut provider_names = HashMap::new();
    for node in nodes.iter() {
        canonical_row_principal("node.node_id", &node.node_id)?;
        canonical_row_principal("node.node_operator_id", &node.node_operator_id)?;
        canonical_row_principal("node.node_provider_id", &node.node_provider_id)?;
        canonical_optional_principal("node.subnet_id", node.subnet_id.as_deref())?;
        canonical_optional_principal(
            "node.cloud_engine_subnet_id",
            node.cloud_engine_subnet_id.as_deref(),
        )?;
        if !seen.insert(node.node_id.as_str()) {
            return invalid_source(format!("duplicate node id {:?}", node.node_id));
        }
        if let Some(expected_name) = provider_names.insert(
            node.node_provider_id.as_str(),
            node.node_provider_name.as_str(),
        ) && expected_name != node.node_provider_name
        {
            return invalid_source(format!(
                "node provider {} has inconsistent names {:?} and {:?}",
                node.node_provider_id, expected_name, node.node_provider_name
            ));
        }
        for (field, value) in [
            ("node.node_type", node.node_type.as_str()),
            ("node.node_reward_type", node.node_reward_type.as_str()),
            ("node.status", node.status.as_str()),
            ("node.data_center_id", node.data_center_id.as_str()),
        ] {
            if value.is_empty() {
                return invalid_source(format!("{field} must not be empty"));
            }
        }
        if node.alert_name.as_deref() == Some("") {
            return invalid_source("node.alert_name must be absent instead of empty");
        }
        if node.subnet_id.is_some()
            && matches!(node.node_type.as_str(), "UNASSIGNED" | "API_BOUNDARY")
        {
            return invalid_source(format!(
                "node {} has assigned subnet evidence but node_type is {}",
                node.node_id, node.node_type
            ));
        }
        if node.subnet_id.is_none() && node.node_type == "REPLICA" {
            return invalid_source(format!(
                "node {} has node_type REPLICA but no subnet_id",
                node.node_id
            ));
        }
    }
    nodes.sort_unstable_by(|left, right| left.node_id.cmp(&right.node_id));
    Ok(())
}

fn canonical_row_principal(field: &'static str, value: &str) -> Result<(), IcHostError> {
    let canonical = canonical_request_principal(field, value).map_err(|error| {
        invalid_source_value(format!(
            "{field} is not a valid canonical principal: {error}"
        ))
    })?;
    if canonical != value {
        return invalid_source(format!("{field} is {value:?}, expected {canonical:?}"));
    }
    Ok(())
}

fn canonical_optional_principal(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), IcHostError> {
    if let Some(value) = value {
        canonical_row_principal(field, value)?;
    }
    Ok(())
}
