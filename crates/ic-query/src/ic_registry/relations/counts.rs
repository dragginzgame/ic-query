#[cfg(feature = "nns-host")]
use crate::ic_registry::normalized_data_center_id;
use crate::ic_registry::{RegistryFetchError, principal_text_from_required_raw};
use ic_host_tools::registry::proto::{NodeOperatorRecord, NodeRecord};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::ic_registry) fn node_operator_principals_from_records(
    node_records: BTreeMap<String, NodeRecord>,
) -> Result<BTreeMap<String, String>, RegistryFetchError> {
    node_records
        .into_iter()
        .map(|(node_principal, record)| {
            if record.node_operator_id.is_empty() {
                return Err(RegistryFetchError::MissingNodeOperatorPrincipal { node_principal });
            }
            let operator = principal_text_from_required_raw(
                &record.node_operator_id,
                "node_record.node_operator_id",
            )?;
            Ok((node_principal, operator))
        })
        .collect()
}

pub(in crate::ic_registry) fn node_operator_references(
    node_operator_principals: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<String>> {
    let mut references = BTreeMap::<String, Vec<String>>::new();
    for (node, operator) in node_operator_principals {
        references
            .entry(operator.clone())
            .or_default()
            .push(node.clone());
    }
    references
}

#[cfg(feature = "nns-host")]
pub(in crate::ic_registry) fn node_counts_from_relations(
    node_principals: &BTreeSet<String>,
    node_operator_principals: &BTreeMap<String, String>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
    key: impl Fn(ResolvedNodeRelation) -> Option<String>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for node_principal in node_principals {
        let relation = resolve_node_relation(
            node_principal,
            node_operator_principals,
            node_operator_records,
        )?;
        if let Some(key) = key(relation) {
            let count = counts.entry(key).or_default();
            *count = count.saturating_add(1);
        }
    }
    Ok(counts)
}

pub(in crate::ic_registry) fn resolved_node_relations(
    node_principals: &BTreeSet<String>,
    node_operator_principals: &BTreeMap<String, String>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, ResolvedNodeRelation>, RegistryFetchError> {
    node_principals
        .iter()
        .map(|node_principal| {
            let relation = resolve_node_relation(
                node_principal,
                node_operator_principals,
                node_operator_records,
            )?;
            Ok((node_principal.clone(), relation))
        })
        .collect()
}

fn resolve_node_relation(
    node_principal: &str,
    node_operator_principals: &BTreeMap<String, String>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<ResolvedNodeRelation, RegistryFetchError> {
    let node_operator_principal =
        node_operator_principals
            .get(node_principal)
            .ok_or_else(|| RegistryFetchError::MissingNodeRecord {
                node_principal: node_principal.to_string(),
            })?;
    let node_operator_record = node_operator_records
        .get(node_operator_principal)
        .ok_or_else(|| RegistryFetchError::MissingNodeOperatorRecord {
            referencing_node_principals: node_operator_principals
                .iter()
                .filter(|(_, operator)| *operator == node_operator_principal)
                .map(|(node, _)| node.clone())
                .collect(),
            node_operator_principal: node_operator_principal.clone(),
        })?;
    let node_provider_principal =
        node_provider_principal_from_record(node_operator_principal, node_operator_record)?;
    Ok(ResolvedNodeRelation {
        #[cfg(feature = "nns-host")]
        node_operator_principal: node_operator_principal.clone(),
        node_provider_principal,
        #[cfg(feature = "nns-host")]
        data_center_id: node_operator_record.dc_id.clone(),
    })
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

///
/// DataCenterOperatorCounts
///
/// Operator totals and distinct provider totals from one admitted data-center relation set.
///

#[cfg(feature = "nns-host")]
pub(in crate::ic_registry) struct DataCenterOperatorCounts {
    pub(in crate::ic_registry) operators: u32,
    pub(in crate::ic_registry) providers: u32,
}

#[cfg(feature = "nns-host")]
pub(in crate::ic_registry) fn data_center_operator_counts_from_records(
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, DataCenterOperatorCounts>, RegistryFetchError> {
    let mut relations = BTreeMap::<String, (u32, BTreeSet<String>)>::new();
    for record in node_operator_records.values() {
        let Some(data_center_id) = normalized_data_center_id(&record.dc_id) else {
            continue;
        };
        let provider = principal_text_from_required_raw(
            &record.node_provider_principal_id,
            "node_operator_record.node_provider_principal_id",
        )?;
        let (operators, providers) = relations.entry(data_center_id).or_default();
        *operators = operators.saturating_add(1);
        providers.insert(provider);
    }
    Ok(relations
        .into_iter()
        .map(|(id, (operators, providers))| {
            (
                id,
                DataCenterOperatorCounts {
                    operators,
                    providers: u32::try_from(providers.len()).unwrap_or(u32::MAX),
                },
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
    #[cfg(feature = "nns-host")]
    pub(in crate::ic_registry) node_operator_principal: String,
    pub(in crate::ic_registry) node_provider_principal: String,
    #[cfg(feature = "nns-host")]
    pub(in crate::ic_registry) data_center_id: String,
}
