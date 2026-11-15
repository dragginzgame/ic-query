//! Join only admitted relationships, retaining every selected node in coverage.

use std::collections::BTreeMap;

use ic_platform_types::Principal;

use super::{
    DataCenterReport, DistributionSummary, GroupCount, NodeRelationships, OperatorReport,
    RecordRead, Relationship, UnavailableReason,
};
use crate::node::NodeReport;

fn unavailable<T>(reason: UnavailableReason) -> Relationship<T> {
    Relationship::Unavailable { reason }
}

fn reference<T>(identity: Option<T>, byte_length: usize) -> Relationship<T> {
    match identity {
        Some(value) => Relationship::Resolved { value },
        None => unavailable(if byte_length == 0 {
            UnavailableReason::MissingReference
        } else {
            UnavailableReason::InvalidReference
        }),
    }
}

pub(super) fn join(
    nodes: &[RecordRead<Principal, NodeReport>],
    operators: &[RecordRead<Principal, OperatorReport>],
    data_centers: &[RecordRead<String, DataCenterReport>],
) -> (Vec<NodeRelationships>, DistributionSummary) {
    let operators: BTreeMap<_, _> = operators
        .iter()
        .filter_map(|row| row.outcome.report().map(|report| (row.id, report)))
        .collect();
    let data_centers: BTreeMap<_, _> = data_centers
        .iter()
        .filter_map(|row| row.outcome.report().map(|report| (row.id.as_str(), report)))
        .collect();
    let mut relationships = Vec::with_capacity(nodes.len());
    let mut providers = BTreeMap::new();
    let mut locations = BTreeMap::new();
    let mut regions = BTreeMap::new();
    let mut owners = BTreeMap::new();
    let mut references_resolved = 0;
    for row in nodes {
        let operator = match row.outcome.report() {
            Some(node) => operators
                .get(&node.node_operator_id)
                .copied()
                .ok_or(UnavailableReason::OperatorRecord),
            None => Err(UnavailableReason::NodeRecord),
        };
        let (provider, data_center_reference, location) = match operator {
            Err(reason) => (
                unavailable(reason),
                unavailable(reason),
                unavailable(reason),
            ),
            Ok(operator) => {
                let provider = reference(operator.provider.identity, operator.provider.byte_length);
                if let Relationship::Resolved { value } = &provider {
                    *providers.entry(*value).or_insert(0) += 1;
                }
                let dc = reference(
                    operator.data_center.identity.clone(),
                    operator.data_center.byte_length,
                );
                let location = match &dc {
                    Relationship::Unavailable { reason } => unavailable(*reason),
                    Relationship::Resolved { value } => {
                        references_resolved += 1;
                        if let Some(record) = data_centers.get(value.as_str()) {
                            *locations.entry(value.clone()).or_insert(0) += 1;
                            if !record.region.is_empty() {
                                *regions.entry(record.region.clone()).or_insert(0) += 1;
                            }
                            if !record.owner.is_empty() {
                                *owners.entry(record.owner.clone()).or_insert(0) += 1;
                            }
                            Relationship::Resolved {
                                value: value.clone(),
                            }
                        } else {
                            unavailable(UnavailableReason::DataCenterRecord)
                        }
                    }
                };
                (provider, dc, location)
            }
        };
        relationships.push(NodeRelationships {
            node_id: row.id,
            provider,
            data_center_reference,
            location,
        });
    }
    let provider_resolved = providers.values().sum();
    let location_resolved = locations.values().sum();
    let region_resolved: usize = regions.values().sum();
    let owner_resolved: usize = owners.values().sum();
    (
        relationships,
        DistributionSummary {
            selected_nodes: nodes.len(),
            provider_resolved,
            provider_unknown: nodes.len() - provider_resolved,
            data_center_references_resolved: references_resolved,
            location_resolved,
            location_unknown: nodes.len() - location_resolved,
            providers: counts(providers),
            data_centers: counts(locations),
            regions: counts(regions),
            region_unknown: nodes.len() - region_resolved,
            owners: counts(owners),
            owner_unknown: nodes.len() - owner_resolved,
        },
    )
}

fn counts<T>(groups: BTreeMap<T, usize>) -> Vec<GroupCount<T>> {
    groups
        .into_iter()
        .map(|(value, nodes)| GroupCount { value, nodes })
        .collect()
}
