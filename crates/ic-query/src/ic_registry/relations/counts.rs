use crate::ic_registry::{
    RegistryFetchError, normalized_data_center_id, principal_text_from_required_raw,
    proto::{NodeOperatorRecord, NodeRecord},
};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::ic_registry) fn node_provider_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in assigned_node_relations(node_principals, node_records, node_operator_records)? {
        let count = counts.entry(relation.node_provider_principal).or_default();
        *count = count.saturating_add(1);
    }
    Ok(counts)
}

pub(in crate::ic_registry) fn node_operator_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in assigned_node_relations(node_principals, node_records, node_operator_records)? {
        let count = counts.entry(relation.node_operator_principal).or_default();
        *count = count.saturating_add(1);
    }
    Ok(counts)
}

pub(in crate::ic_registry) fn data_center_node_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in assigned_node_relations(node_principals, node_records, node_operator_records)? {
        if let Some(data_center_id) = relation.data_center_id {
            let count = counts.entry(data_center_id).or_default();
            *count = count.saturating_add(1);
        }
    }
    Ok(counts)
}

fn assigned_node_relations(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<Vec<AssignedNodeRelation>, RegistryFetchError> {
    let mut relations = Vec::with_capacity(node_principals.len());
    for node_principal in node_principals {
        let node_record =
            node_records
                .get(node_principal)
                .ok_or(RegistryFetchError::MissingField {
                    field: "node_record",
                })?;
        let node_operator_principal = principal_text_from_required_raw(
            &node_record.node_operator_id,
            "node_record.node_operator_id",
        )?;
        let node_operator_record = node_operator_records.get(&node_operator_principal).ok_or(
            RegistryFetchError::MissingField {
                field: "node_operator_record",
            },
        )?;
        let node_provider_principal = principal_text_from_required_raw(
            &node_operator_record.node_provider_principal_id,
            "node_operator_record.node_provider_principal_id",
        )?;
        relations.push(AssignedNodeRelation {
            node_operator_principal,
            node_provider_principal,
            data_center_id: normalized_data_center_id(&node_operator_record.dc_id),
        });
    }
    Ok(relations)
}

pub(in crate::ic_registry) fn data_center_operator_counts_from_records(
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::<String, u32>::new();
    for record in node_operator_records.values() {
        if let Some(data_center_id) = normalized_data_center_id(&record.dc_id) {
            let count = counts.entry(data_center_id).or_default();
            *count = count.saturating_add(1);
        }
    }
    counts
}

pub(in crate::ic_registry) fn data_center_provider_counts_from_records(
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut providers_by_data_center = BTreeMap::<String, BTreeSet<String>>::new();
    for record in node_operator_records.values() {
        let Some(data_center_id) = normalized_data_center_id(&record.dc_id) else {
            continue;
        };
        let node_provider_principal = principal_text_from_required_raw(
            &record.node_provider_principal_id,
            "node_operator_record.node_provider_principal_id",
        )?;
        providers_by_data_center
            .entry(data_center_id)
            .or_default()
            .insert(node_provider_principal);
    }
    Ok(providers_by_data_center
        .into_iter()
        .map(|(data_center_id, providers)| {
            (
                data_center_id,
                u32::try_from(providers.len()).unwrap_or(u32::MAX),
            )
        })
        .collect())
}

struct AssignedNodeRelation {
    node_operator_principal: String,
    node_provider_principal: String,
    data_center_id: Option<String>,
}
