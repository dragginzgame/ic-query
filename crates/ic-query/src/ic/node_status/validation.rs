//! Module: ic::node_status::validation
//!
//! Responsibility: shared observed provenance, node-row, and scope validation.
//! Does not own: source calls, cache policy, projections, or rendering.
//! Boundary: canonicalizes new source rows but only validates persisted/report rows.

use super::{
    IC_NODE_STATUS_SCHEMA_VERSION, IcNodeStatusObservation, IcNodeStatusRow, IcNodeStatusScope,
    MAX_IC_NODE_STATUS_ROWS,
};
use crate::{
    http_endpoint::parse_http_endpoint,
    ic::IC_DASHBOARD_AUTHORITY,
    subnet_catalog::{MAINNET_NETWORK, parse_utc_timestamp_secs},
};
use candid::Principal;
use std::collections::HashMap;
#[cfg(feature = "dashboard-host")]
use std::collections::HashSet;

pub(super) fn validate_node_status_observation(
    observation: &IcNodeStatusObservation,
) -> Result<u64, String> {
    let source = &observation.source;
    if source.schema_version != IC_NODE_STATUS_SCHEMA_VERSION {
        return Err(format!(
            "schema_version is {}, expected {IC_NODE_STATUS_SCHEMA_VERSION}",
            source.schema_version
        ));
    }
    if source.network != MAINNET_NETWORK {
        return Err(format!(
            "network is {:?}, expected {MAINNET_NETWORK:?}",
            source.network
        ));
    }
    if source.certified || source.point_in_time_guaranteed {
        return Err(
            "Dashboard node observations cannot claim certification or point-in-time guarantees"
                .to_string(),
        );
    }
    if source.authority != IC_DASHBOARD_AUTHORITY {
        return Err(format!(
            "authority is {:?}, expected {IC_DASHBOARD_AUTHORITY:?}",
            source.authority
        ));
    }
    if observation.scope != IcNodeStatusScope::DashboardMainnetDefault
        || observation.cloud_engine_nodes_included
    {
        return Err(
            "observation does not describe the Dashboard default mainnet node scope".to_string(),
        );
    }
    if source.source_endpoint.is_empty() || source.fetched_by.is_empty() {
        return Err("source_endpoint and fetched_by must not be empty".to_string());
    }
    parse_http_endpoint(&source.source_endpoint)
        .map_err(|reason| format!("invalid source_endpoint: {reason}"))?;
    parse_utc_timestamp_secs(&source.fetched_at)
        .ok_or_else(|| "fetched_at is not a canonical UTC timestamp".to_string())
}

#[cfg(feature = "dashboard-host")]
pub(in crate::ic) fn canonicalize_node_status_rows(
    nodes: &mut [IcNodeStatusRow],
) -> Result<(), String> {
    canonicalize_node_status_rows_with_policy(nodes, MAX_IC_NODE_STATUS_ROWS, true)
}

#[cfg(feature = "dashboard-host")]
/// Validate and canonicalize raw node rows under an explicit collection policy.
pub fn canonicalize_node_status_rows_with_policy(
    nodes: &mut [IcNodeStatusRow],
    max_rows: u32,
    require_nonempty: bool,
) -> Result<(), String> {
    validate_node_status_rows(nodes, max_rows, require_nonempty)?;
    let mut seen = HashSet::with_capacity(nodes.len());
    for node in nodes.iter() {
        if !seen.insert(node.node_id.as_str()) {
            return Err(format!("duplicate node id {:?}", node.node_id));
        }
    }
    nodes.sort_unstable_by(|left, right| left.node_id.cmp(&right.node_id));
    Ok(())
}

pub(in crate::ic) fn validate_canonical_node_status_rows(
    nodes: &[IcNodeStatusRow],
) -> Result<(), String> {
    validate_node_status_rows(nodes, MAX_IC_NODE_STATUS_ROWS, true)?;
    if nodes
        .windows(2)
        .any(|pair| pair[0].node_id >= pair[1].node_id)
    {
        return Err("node rows are not in strict canonical node-id order".to_string());
    }
    Ok(())
}

pub(in crate::ic) fn validate_default_node_scope(nodes: &[IcNodeStatusRow]) -> Result<(), String> {
    if let Some(node) = nodes
        .iter()
        .find(|node| node.cloud_engine_subnet_id.is_some())
    {
        return Err(format!(
            "node {} contains cloud-engine Subnet evidence in the default public-mainnet scope",
            node.node_id
        ));
    }
    Ok(())
}

fn validate_node_status_rows(
    nodes: &[IcNodeStatusRow],
    max_rows: u32,
    require_nonempty: bool,
) -> Result<(), String> {
    if require_nonempty && nodes.is_empty() {
        return Err("mainnet node snapshot must contain at least one row".to_string());
    }
    if u32::try_from(nodes.len()).unwrap_or(u32::MAX) > max_rows {
        return Err(format!(
            "source returned {} node rows; maximum is {max_rows}",
            nodes.len()
        ));
    }

    let mut provider_names = HashMap::new();
    for node in nodes {
        canonical_row_principal("node.node_id", &node.node_id)?;
        canonical_row_principal("node.node_operator_id", &node.node_operator_id)?;
        canonical_row_principal("node.node_provider_id", &node.node_provider_id)?;
        canonical_optional_principal("node.subnet_id", node.subnet_id.as_deref())?;
        canonical_optional_principal(
            "node.cloud_engine_subnet_id",
            node.cloud_engine_subnet_id.as_deref(),
        )?;
        if let Some(expected_name) = provider_names.insert(
            node.node_provider_id.as_str(),
            node.node_provider_name.as_str(),
        ) && expected_name != node.node_provider_name
        {
            return Err(format!(
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
                return Err(format!("{field} must not be empty"));
            }
        }
        if node.alert_name.as_deref() == Some("") {
            return Err("node.alert_name must be absent instead of empty".to_string());
        }
        if node.subnet_id.is_some()
            && matches!(node.node_type.as_str(), "UNASSIGNED" | "API_BOUNDARY")
        {
            return Err(format!(
                "node {} has assigned subnet evidence but node_type is {}",
                node.node_id, node.node_type
            ));
        }
        if node.subnet_id.is_none() && node.node_type == "REPLICA" {
            return Err(format!(
                "node {} has node_type REPLICA but no subnet_id",
                node.node_id
            ));
        }
    }
    Ok(())
}

fn canonical_row_principal(field: &'static str, value: &str) -> Result<(), String> {
    let canonical = Principal::from_text(value)
        .map(|principal| principal.to_text())
        .map_err(|error| format!("{field} is not a valid canonical principal: {error}"))?;
    if canonical != value {
        return Err(format!("{field} is {value:?}, expected {canonical:?}"));
    }
    Ok(())
}

fn canonical_optional_principal(field: &'static str, value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        canonical_row_principal(field, value)?;
    }
    Ok(())
}
