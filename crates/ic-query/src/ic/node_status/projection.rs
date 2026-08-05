//! Module: ic::node_status::projection
//!
//! Responsibility: pure node, Subnet, and provider projections from one status snapshot.
//! Does not own: live calls, cache IO, wire decoding, or text rendering.
//! Boundary: keeps targets and attention filters as views over one collected identity.

use super::{
    IcNodeProviderStatusReport, IcNodeProviderStatusRow, IcNodeStatusCounts,
    IcNodeStatusProjectionError, IcNodeStatusReport, IcNodeStatusRow, IcNodeStatusScope,
    IcNodeStatusSnapshot, IcNodeStatusView, IcSubnetStatusReport, IcSubnetStatusRow,
    node_status_counts, node_status_group_counts, validate_canonical_node_status_rows,
    validate_default_node_scope,
};
use std::collections::BTreeMap;

/// Project a node-level status report from one complete observed snapshot.
pub fn ic_node_status_report_from_snapshot(
    snapshot: &IcNodeStatusSnapshot,
    view: &IcNodeStatusView,
) -> Result<IcNodeStatusReport, IcNodeStatusProjectionError> {
    validate_snapshot(snapshot)?;
    let resolution = resolve_optional_target(
        "node",
        view.target.as_deref(),
        snapshot.nodes.iter().map(|node| node.node_id.as_str()),
        "node_principal",
    )?;
    let nodes = snapshot
        .nodes
        .iter()
        .filter(|node| {
            resolution.as_ref().map_or_else(
                || view.include_all || node.is_non_up(),
                |resolution| node.node_id == resolution.resolved,
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    Ok(IcNodeStatusReport {
        observation: snapshot.observation.clone(),
        snapshot_node_count: snapshot.node_count,
        counts: snapshot.counts.clone(),
        include_all: view.include_all,
        requested_target: view.target.clone(),
        resolved_target: resolution.as_ref().map(|value| value.resolved.clone()),
        resolved_from: resolution.map(|value| value.resolved_from),
        returned_node_count: nodes.len(),
        nodes,
    })
}

/// Project a Subnet-level status report from one complete observed snapshot.
pub fn ic_subnet_status_report_from_snapshot(
    snapshot: &IcNodeStatusSnapshot,
    view: &IcNodeStatusView,
) -> Result<IcSubnetStatusReport, IcNodeStatusProjectionError> {
    validate_snapshot(snapshot)?;
    let mut grouped = BTreeMap::<String, Vec<&IcNodeStatusRow>>::new();
    for node in &snapshot.nodes {
        if let Some(subnet_id) = &node.subnet_id {
            grouped.entry(subnet_id.clone()).or_default().push(node);
        }
    }
    let resolution = resolve_optional_target(
        "Subnet",
        view.target.as_deref(),
        grouped.keys().map(String::as_str),
        "subnet_principal",
    )?;
    let subnet_count = grouped.len();
    let attention_subnet_count = grouped
        .values()
        .filter(|nodes| counts_for(nodes.iter().copied()).non_up() > 0)
        .count();
    let subnets = grouped
        .into_iter()
        .filter(|(subnet_id, nodes)| {
            resolution.as_ref().map_or_else(
                || view.include_all || counts_for(nodes.iter().copied()).non_up() > 0,
                |resolution| subnet_id == &resolution.resolved,
            )
        })
        .map(|(subnet_id, nodes)| subnet_row(subnet_id, &nodes))
        .collect::<Vec<_>>();

    Ok(IcSubnetStatusReport {
        observation: snapshot.observation.clone(),
        snapshot_node_count: snapshot.node_count,
        assigned_node_count: snapshot.counts.assignment_statuses.assigned.total,
        subnet_count,
        attention_subnet_count,
        include_all: view.include_all,
        requested_target: view.target.clone(),
        resolved_target: resolution.as_ref().map(|value| value.resolved.clone()),
        resolved_from: resolution.map(|value| value.resolved_from),
        returned_subnet_count: subnets.len(),
        subnets,
    })
}

/// Project a node-provider status report from one complete observed snapshot.
pub fn ic_node_provider_status_report_from_snapshot(
    snapshot: &IcNodeStatusSnapshot,
    view: &IcNodeStatusView,
) -> Result<IcNodeProviderStatusReport, IcNodeStatusProjectionError> {
    validate_snapshot(snapshot)?;
    let mut grouped = BTreeMap::<String, Vec<&IcNodeStatusRow>>::new();
    for node in &snapshot.nodes {
        grouped
            .entry(node.node_provider_id.clone())
            .or_default()
            .push(node);
    }
    let resolution = resolve_optional_target(
        "node provider",
        view.target.as_deref(),
        grouped.keys().map(String::as_str),
        "node_provider_principal",
    )?;
    let provider_count = grouped.len();
    let attention_provider_count = grouped
        .values()
        .filter(|nodes| counts_for(nodes.iter().copied()).non_up() > 0)
        .count();
    let providers = grouped
        .into_iter()
        .filter(|(provider_id, nodes)| {
            resolution.as_ref().map_or_else(
                || view.include_all || counts_for(nodes.iter().copied()).non_up() > 0,
                |resolution| provider_id == &resolution.resolved,
            )
        })
        .map(|(provider_id, nodes)| provider_row(provider_id, &nodes))
        .collect::<Vec<_>>();

    Ok(IcNodeProviderStatusReport {
        observation: snapshot.observation.clone(),
        snapshot_node_count: snapshot.node_count,
        provider_count,
        attention_provider_count,
        include_all: view.include_all,
        requested_target: view.target.clone(),
        resolved_target: resolution.as_ref().map(|value| value.resolved.clone()),
        resolved_from: resolution.map(|value| value.resolved_from),
        returned_provider_count: providers.len(),
        providers,
    })
}

fn subnet_row(subnet_id: String, nodes: &[&IcNodeStatusRow]) -> IcSubnetStatusRow {
    let statuses = counts_for(nodes.iter().copied());
    let fault_tolerance_node_count = statuses.total.saturating_sub(1) / 3;
    let first_exceeding_count = fault_tolerance_node_count.saturating_add(1);
    let non_up_nodes = nodes
        .iter()
        .filter(|node| node.is_non_up())
        .map(|node| (*node).clone())
        .collect();
    IcSubnetStatusRow {
        subnet_id,
        additional_down_nodes_to_exceed_fault_tolerance: first_exceeding_count
            .saturating_sub(statuses.down),
        additional_non_up_nodes_to_exceed_fault_tolerance: first_exceeding_count
            .saturating_sub(statuses.non_up()),
        down_fault_tolerance_exceeded: statuses.down > fault_tolerance_node_count,
        conservative_non_up_fault_tolerance_exceeded: statuses.non_up()
            > fault_tolerance_node_count,
        fault_tolerance_node_count,
        statuses,
        non_up_nodes,
    }
}

fn provider_row(node_provider_id: String, nodes: &[&IcNodeStatusRow]) -> IcNodeProviderStatusRow {
    let node_provider_name = nodes
        .first()
        .map_or_else(String::new, |node| node.node_provider_name.clone());
    let non_up_nodes = nodes
        .iter()
        .filter(|node| node.is_non_up())
        .map(|node| (*node).clone())
        .collect();
    IcNodeProviderStatusRow {
        node_provider_id,
        node_provider_name,
        counts: node_status_group_counts(nodes.iter().copied()),
        non_up_nodes,
    }
}

fn counts_for<'a>(nodes: impl Iterator<Item = &'a IcNodeStatusRow>) -> IcNodeStatusCounts {
    node_status_counts(nodes)
}

fn validate_snapshot(snapshot: &IcNodeStatusSnapshot) -> Result<(), IcNodeStatusProjectionError> {
    if snapshot.observation.scope != IcNodeStatusScope::DashboardMainnetDefault
        || snapshot.observation.cloud_engine_nodes_included
    {
        return invalid_snapshot(
            "snapshot does not describe the Dashboard default mainnet node scope",
        );
    }
    validate_canonical_node_status_rows(&snapshot.nodes)
        .map_err(|reason| IcNodeStatusProjectionError::InvalidSnapshot { reason })?;
    validate_default_node_scope(&snapshot.nodes)
        .map_err(|reason| IcNodeStatusProjectionError::InvalidSnapshot { reason })?;
    if snapshot.node_count != snapshot.nodes.len() {
        return invalid_snapshot(format!(
            "node_count is {}, actual row count is {}",
            snapshot.node_count,
            snapshot.nodes.len()
        ));
    }
    let counts = node_status_group_counts(snapshot.nodes.iter());
    if snapshot.counts != counts {
        return invalid_snapshot("counts do not match raw node rows");
    }
    Ok(())
}

fn invalid_snapshot<T>(reason: impl Into<String>) -> Result<T, IcNodeStatusProjectionError> {
    Err(IcNodeStatusProjectionError::InvalidSnapshot {
        reason: reason.into(),
    })
}

struct TargetResolution {
    resolved: String,
    resolved_from: String,
}

fn resolve_optional_target<'a>(
    kind: &'static str,
    target: Option<&str>,
    identifiers: impl Iterator<Item = &'a str>,
    exact_label: &'static str,
) -> Result<Option<TargetResolution>, IcNodeStatusProjectionError> {
    let Some(target) = target else {
        return Ok(None);
    };
    let target = target.trim();
    if target.is_empty() {
        return Err(IcNodeStatusProjectionError::EmptyTarget { kind });
    }
    let identifiers = identifiers.collect::<Vec<_>>();
    if identifiers.contains(&target) {
        return Ok(Some(TargetResolution {
            resolved: target.to_string(),
            resolved_from: exact_label.to_string(),
        }));
    }
    let matches = identifiers
        .into_iter()
        .filter(|identifier| identifier.starts_with(target))
        .map(str::to_string)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Err(IcNodeStatusProjectionError::UnknownTarget {
            kind,
            target: target.to_string(),
        }),
        [resolved] => Ok(Some(TargetResolution {
            resolved: resolved.clone(),
            resolved_from: format!("{exact_label}_prefix"),
        })),
        _ => Err(IcNodeStatusProjectionError::AmbiguousTarget {
            kind,
            prefix: target.to_string(),
            matches,
        }),
    }
}
