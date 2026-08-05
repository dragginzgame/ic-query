//! Module: ic::node_status::validation
//!
//! Responsibility: shared observed node-row and scope validation.
//! Does not own: source calls, cache policy, projections, or rendering.
//! Boundary: canonicalizes new source rows but only validates persisted/report rows.

use super::{IcNodeStatusRow, MAX_IC_NODE_STATUS_ROWS};
use candid::Principal;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "host")]
pub(in crate::ic) fn canonicalize_node_status_rows(
    nodes: &mut [IcNodeStatusRow],
) -> Result<(), String> {
    validate_node_status_rows(nodes)?;
    nodes.sort_unstable_by(|left, right| left.node_id.cmp(&right.node_id));
    Ok(())
}

pub(in crate::ic) fn validate_canonical_node_status_rows(
    nodes: &[IcNodeStatusRow],
) -> Result<(), String> {
    validate_node_status_rows(nodes)?;
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

fn validate_node_status_rows(nodes: &[IcNodeStatusRow]) -> Result<(), String> {
    if nodes.is_empty() {
        return Err("mainnet node snapshot must contain at least one row".to_string());
    }
    if u32::try_from(nodes.len()).unwrap_or(u32::MAX) > MAX_IC_NODE_STATUS_ROWS {
        return Err(format!(
            "source returned {} node rows; maximum is {MAX_IC_NODE_STATUS_ROWS}",
            nodes.len()
        ));
    }

    let mut seen = HashSet::with_capacity(nodes.len());
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
        if !seen.insert(node.node_id.as_str()) {
            return Err(format!("duplicate node id {:?}", node.node_id));
        }
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
