#[cfg(feature = "host")]
use crate::ic_registry::normalized_data_center_id;
use crate::ic_registry::{
    RegistryFetchError, principal_text_from_required_raw,
    proto::{NodeOperatorRecord, NodeRecord},
};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::ic_registry) fn node_operator_references_from_records(
    node_records: &BTreeMap<String, NodeRecord>,
) -> Result<BTreeMap<String, Vec<String>>, RegistryFetchError> {
    let mut references = BTreeMap::<String, Vec<String>>::new();
    for (node_principal, record) in node_records {
        if record.node_operator_id.is_empty() {
            return Err(RegistryFetchError::MissingNodeOperatorPrincipal {
                node_principal: node_principal.clone(),
            });
        }
        let node_operator_principal = principal_text_from_required_raw(
            &record.node_operator_id,
            "node_record.node_operator_id",
        )?;
        references
            .entry(node_operator_principal)
            .or_default()
            .push(node_principal.clone());
    }
    Ok(references)
}

#[cfg(feature = "host")]
pub(in crate::ic_registry) fn node_provider_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in
        resolved_node_relations_from_records(node_principals, node_records, node_operator_records)?
            .into_values()
    {
        let count = counts.entry(relation.node_provider_principal).or_default();
        *count = count.saturating_add(1);
    }
    Ok(counts)
}

#[cfg(feature = "host")]
pub(in crate::ic_registry) fn node_operator_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in
        resolved_node_relations_from_records(node_principals, node_records, node_operator_records)?
            .into_values()
    {
        let count = counts.entry(relation.node_operator_principal).or_default();
        *count = count.saturating_add(1);
    }
    Ok(counts)
}

#[cfg(feature = "host")]
pub(in crate::ic_registry) fn data_center_node_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in
        resolved_node_relations_from_records(node_principals, node_records, node_operator_records)?
            .into_values()
    {
        if let Some(data_center_id) = normalized_data_center_id(&relation.data_center_id) {
            let count = counts.entry(data_center_id).or_default();
            *count = count.saturating_add(1);
        }
    }
    Ok(counts)
}

pub(in crate::ic_registry) fn resolved_node_relations_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, ResolvedNodeRelation>, RegistryFetchError> {
    let node_operator_references = node_operator_references_from_records(node_records)?;
    let mut relations = BTreeMap::new();
    for node_principal in node_principals {
        let node_record = node_records.get(node_principal).ok_or_else(|| {
            RegistryFetchError::MissingNodeRecord {
                node_principal: node_principal.clone(),
            }
        })?;
        let node_operator_principal = principal_text_from_required_raw(
            &node_record.node_operator_id,
            "node_record.node_operator_id",
        )?;
        let referencing_node_principals = node_operator_references
            .get(&node_operator_principal)
            .cloned()
            .ok_or(RegistryFetchError::MissingField {
                field: "node_operator_references",
            })?;
        let node_operator_record = node_operator_records
            .get(&node_operator_principal)
            .ok_or_else(|| RegistryFetchError::MissingNodeOperatorRecord {
                referencing_node_principals,
                node_operator_principal: node_operator_principal.clone(),
            })?;
        let node_provider_principal =
            node_provider_principal_from_record(&node_operator_principal, node_operator_record)?;
        relations.insert(
            node_principal.clone(),
            ResolvedNodeRelation {
                #[cfg(feature = "host")]
                node_operator_principal,
                node_provider_principal,
                #[cfg(feature = "host")]
                data_center_id: node_operator_record.dc_id.clone(),
            },
        );
    }
    Ok(relations)
}

pub(in crate::ic_registry) fn node_provider_principal_from_record(
    node_operator_principal: &str,
    record: &NodeOperatorRecord,
) -> Result<String, RegistryFetchError> {
    if record.node_provider_principal_id.is_empty() {
        return Err(RegistryFetchError::MissingNodeProviderPrincipal {
            node_operator_principal: node_operator_principal.to_string(),
        });
    }
    principal_text_from_required_raw(
        &record.node_provider_principal_id,
        "node_operator_record.node_provider_principal_id",
    )
}

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
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

///
/// ResolvedNodeRelation
///
/// Canonical operator, provider, and data-center relation for one assigned node.
///

pub(in crate::ic_registry) struct ResolvedNodeRelation {
    #[cfg(feature = "host")]
    pub(in crate::ic_registry) node_operator_principal: String,
    pub(in crate::ic_registry) node_provider_principal: String,
    #[cfg(feature = "host")]
    pub(in crate::ic_registry) data_center_id: String,
}
