//! Module: ic::node_status::projection
//!
//! Responsibility: pure node, Subnet, and provider projections from one status snapshot.
//! Does not own: live calls, cache IO, wire decoding, or text rendering.
//! Boundary: keeps targets and attention filters as views over one collected identity.

use super::{
    IcNodeCountComparison, IcNodeCountComparisonCounts, IcNodeProviderStatusReport,
    IcNodeProviderStatusRow, IcNodeStatusCounts, IcNodeStatusGroupCounts,
    IcNodeStatusProjectionError, IcNodeStatusReport, IcNodeStatusRow, IcNodeStatusSnapshot,
    IcNodeStatusView, IcSubnetStatusReport, IcSubnetStatusRow, node_status_group_counts,
    validate_canonical_node_status_rows, validate_default_node_scope,
    validation::validate_node_status_observation,
};
use crate::identifier_match::{IdentifierMatch, match_identifier};
use std::collections::BTreeMap;

/// Project a node-level status report from one complete observed snapshot.
pub fn ic_node_status_report_from_snapshot(
    snapshot: &IcNodeStatusSnapshot,
    view: &IcNodeStatusView,
) -> Result<IcNodeStatusReport, IcNodeStatusProjectionError> {
    ValidatedNodeStatusSnapshot::new(snapshot)?.node_report(view)
}

/// Project a Subnet-level status report from one complete observed snapshot.
pub fn ic_subnet_status_report_from_snapshot(
    snapshot: &IcNodeStatusSnapshot,
    view: &IcNodeStatusView,
) -> Result<IcSubnetStatusReport, IcNodeStatusProjectionError> {
    ValidatedNodeStatusSnapshot::new(snapshot)?.subnet_report(view)
}

/// Project a node-provider status report from one complete observed snapshot.
pub fn ic_node_provider_status_report_from_snapshot(
    snapshot: &IcNodeStatusSnapshot,
    view: &IcNodeStatusView,
) -> Result<IcNodeProviderStatusReport, IcNodeStatusProjectionError> {
    ValidatedNodeStatusSnapshot::new(snapshot)?.provider_report(view)
}

///
/// ValidatedNodeStatusSnapshot
///
/// Immutable projection input admitted through raw-snapshot or cache validation.
///

pub(super) struct ValidatedNodeStatusSnapshot<'a> {
    snapshot: &'a IcNodeStatusSnapshot,
}

impl<'a> ValidatedNodeStatusSnapshot<'a> {
    fn new(snapshot: &'a IcNodeStatusSnapshot) -> Result<Self, IcNodeStatusProjectionError> {
        validate_snapshot(snapshot)?;
        Ok(Self { snapshot })
    }

    #[cfg(feature = "dashboard-host")]
    pub(super) const fn from_cache(cached: &'a super::cache::LoadedNodeStatusSnapshot) -> Self {
        Self {
            snapshot: cached.snapshot(),
        }
    }

    pub(super) fn node_report(
        &self,
        view: &IcNodeStatusView,
    ) -> Result<IcNodeStatusReport, IcNodeStatusProjectionError> {
        let snapshot = self.snapshot;
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
                view_includes(view, resolution.as_ref(), &node.node_id, node.is_non_up())
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
    pub(super) fn subnet_report(
        &self,
        view: &IcNodeStatusView,
    ) -> Result<IcSubnetStatusReport, IcNodeStatusProjectionError> {
        let snapshot = self.snapshot;
        let grouped = group_nodes(&snapshot.nodes, |node| node.subnet_id.as_deref());
        let resolution = resolve_optional_target(
            "Subnet",
            view.target.as_deref(),
            grouped.keys().copied(),
            "subnet_principal",
        )?;
        let subnet_count = grouped.len();
        let mut attention_subnet_count = 0;
        let mut subnets = Vec::new();
        for (subnet_id, group) in grouped {
            let statuses = group.counts.statuses;
            let requires_attention = statuses.non_up() > 0;
            attention_subnet_count += usize::from(requires_attention);
            if view_includes(view, resolution.as_ref(), subnet_id, requires_attention) {
                subnets.push(subnet_row(
                    subnet_id.to_string(),
                    statuses,
                    group.non_up_nodes.into_iter().cloned().collect(),
                ));
            }
        }

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
    pub(super) fn provider_report(
        &self,
        view: &IcNodeStatusView,
    ) -> Result<IcNodeProviderStatusReport, IcNodeStatusProjectionError> {
        let snapshot = self.snapshot;
        let grouped = group_nodes(&snapshot.nodes, |node| Some(node.node_provider_id.as_str()));
        let resolution = resolve_optional_target(
            "node provider",
            view.target.as_deref(),
            grouped.keys().copied(),
            "node_provider_principal",
        )?;
        let provider_count = grouped.len();
        let mut attention_provider_count = 0;
        let mut up_comparisons = IcNodeCountComparisonCounts::default();
        let mut non_up_comparisons = IcNodeCountComparisonCounts::default();
        let mut providers = Vec::new();
        for (node_provider_id, group) in grouped {
            let counts = group.counts;
            let assigned = &counts.assignment_statuses.assigned;
            let unassigned = &counts.assignment_statuses.unassigned;
            let unassigned_up_vs_assigned_up =
                IcNodeCountComparison::from_counts(unassigned.up, assigned.up);
            let unassigned_non_up_vs_assigned_non_up =
                IcNodeCountComparison::from_counts(unassigned.non_up(), assigned.non_up());
            increment_comparison(&mut up_comparisons, unassigned_up_vs_assigned_up);
            increment_comparison(
                &mut non_up_comparisons,
                unassigned_non_up_vs_assigned_non_up,
            );
            let requires_attention = counts.statuses.non_up() > 0;
            attention_provider_count += usize::from(requires_attention);
            if view_includes(
                view,
                resolution.as_ref(),
                node_provider_id,
                requires_attention,
            ) {
                providers.push(IcNodeProviderStatusRow {
                    node_provider_id: node_provider_id.to_string(),
                    node_provider_name: group.first.node_provider_name.clone(),
                    unassigned_up_vs_assigned_up,
                    unassigned_non_up_vs_assigned_non_up,
                    counts,
                    non_up_nodes: group.non_up_nodes.into_iter().cloned().collect(),
                });
            }
        }

        Ok(IcNodeProviderStatusReport {
            observation: snapshot.observation.clone(),
            snapshot_node_count: snapshot.node_count,
            provider_count,
            attention_provider_count,
            unassigned_up_vs_assigned_up_provider_counts: up_comparisons,
            unassigned_non_up_vs_assigned_non_up_provider_counts: non_up_comparisons,
            include_all: view.include_all,
            requested_target: view.target.clone(),
            resolved_target: resolution.as_ref().map(|value| value.resolved.clone()),
            resolved_from: resolution.map(|value| value.resolved_from),
            returned_provider_count: providers.len(),
            providers,
        })
    }
}

const fn subnet_row(
    subnet_id: String,
    statuses: IcNodeStatusCounts,
    non_up_nodes: Vec<IcNodeStatusRow>,
) -> IcSubnetStatusRow {
    let fault_tolerance_node_count = statuses.total.saturating_sub(1) / 3;
    let first_exceeding_count = fault_tolerance_node_count.saturating_add(1);
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

struct NodeStatusGroup<'a> {
    first: &'a IcNodeStatusRow,
    counts: IcNodeStatusGroupCounts,
    non_up_nodes: Vec<&'a IcNodeStatusRow>,
}

fn group_nodes(
    nodes: &[IcNodeStatusRow],
    key: impl Fn(&IcNodeStatusRow) -> Option<&str>,
) -> BTreeMap<&str, NodeStatusGroup<'_>> {
    let mut groups = BTreeMap::new();
    for node in nodes {
        let Some(key) = key(node) else {
            continue;
        };
        let group = groups.entry(key).or_insert_with(|| NodeStatusGroup {
            first: node,
            counts: IcNodeStatusGroupCounts::default(),
            non_up_nodes: Vec::new(),
        });
        super::counts::observe_node_status(&mut group.counts, node);
        if node.is_non_up() {
            group.non_up_nodes.push(node);
        }
    }
    groups
}

const fn increment_comparison(
    counts: &mut IcNodeCountComparisonCounts,
    comparison: IcNodeCountComparison,
) {
    match comparison {
        IcNodeCountComparison::Less => counts.less += 1,
        IcNodeCountComparison::Equal => counts.equal += 1,
        IcNodeCountComparison::Greater => counts.greater += 1,
    }
}

fn validate_snapshot(snapshot: &IcNodeStatusSnapshot) -> Result<(), IcNodeStatusProjectionError> {
    validate_node_status_observation(&snapshot.observation)
        .map_err(|reason| IcNodeStatusProjectionError::InvalidSnapshot { reason })?;
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

fn view_includes(
    view: &IcNodeStatusView,
    resolution: Option<&TargetResolution>,
    identifier: &str,
    requires_attention: bool,
) -> bool {
    resolution.map_or_else(
        || view.include_all || requires_attention,
        |resolution| identifier == resolution.resolved,
    )
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
    match match_identifier(identifiers, |value| value, Some(target), target) {
        IdentifierMatch::Exact(resolved) => Ok(Some(TargetResolution {
            resolved: resolved.to_string(),
            resolved_from: exact_label.to_string(),
        })),
        IdentifierMatch::Prefix(resolved) => Ok(Some(TargetResolution {
            resolved: resolved.to_string(),
            resolved_from: format!("{exact_label}_prefix"),
        })),
        IdentifierMatch::Missing => Err(IcNodeStatusProjectionError::UnknownTarget {
            kind,
            target: target.to_string(),
        }),
        IdentifierMatch::Ambiguous(matches) => Err(IcNodeStatusProjectionError::AmbiguousTarget {
            kind,
            prefix: target.to_string(),
            matches: matches.into_iter().map(str::to_string).collect(),
        }),
    }
}
