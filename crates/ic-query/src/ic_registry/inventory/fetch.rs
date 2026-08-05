#[cfg(feature = "nns-host")]
use super::data_center::fetch_data_center_records_for_inventory;
use super::{
    INVENTORY_FETCH_CONCURRENCY,
    keys::{node_operator_record_key, node_record_key},
};
use crate::ic_registry::{
    RegistryFetchError, SUBNET_LIST_KEY, principal_text_from_raw,
    proto::{NodeOperatorRecord, NodeRecord, SubnetListRecord, SubnetRecord},
    relations::{
        RegistryRelationInventory, RegistryRelationInventoryScope,
        assigned_node_principals_from_subnets, node_operator_references_from_records,
    },
    subnet_record_key,
    transport::{decode_message, get_registry_value},
};
use candid::Principal;
use futures::{StreamExt, TryStreamExt, stream};
use ic_agent::Agent;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "nns-host")]
use crate::ic_registry::relations::node_provider_counts_from_records;

#[cfg(feature = "nns-host")]
pub(in crate::ic_registry) async fn fetch_node_provider_node_counts(
    agent: &Agent,
    registry_canister: &Principal,
    registry_version: u64,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let inventory = fetch_registry_relation_inventory(
        agent,
        registry_canister,
        registry_version,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    node_provider_counts_from_records(
        &inventory.node_principals,
        &inventory.node_records,
        &inventory.node_operator_records,
    )
}

pub(in crate::ic_registry) async fn fetch_registry_relation_inventory(
    agent: &Agent,
    registry_canister: &Principal,
    registry_version: u64,
    scope: RegistryRelationInventoryScope,
) -> Result<RegistryRelationInventory, RegistryFetchError> {
    let subnet_list_bytes =
        get_registry_value(agent, registry_canister, SUBNET_LIST_KEY, registry_version).await?;
    let subnet_list = decode_message::<SubnetListRecord>("SubnetListRecord", &subnet_list_bytes)?;
    if subnet_list.subnets.is_empty() {
        return Err(RegistryFetchError::EmptySubnetList);
    }

    let subnet_principals = subnet_list
        .subnets
        .iter()
        .map(|subnet_raw| principal_text_from_raw(subnet_raw, "subnet_list.subnets"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut unique_subnet_principals = BTreeSet::new();
    for subnet_principal in &subnet_principals {
        if !unique_subnet_principals.insert(subnet_principal.clone()) {
            return Err(RegistryFetchError::DuplicateSubnetPrincipal {
                subnet_principal: subnet_principal.clone(),
            });
        }
    }
    let subnet_records = stream::iter(subnet_principals)
        .map(|subnet_principal| async move {
            let key = subnet_record_key(&subnet_principal);
            let record_bytes =
                get_registry_value(agent, registry_canister, &key, registry_version).await?;
            let record = decode_message::<SubnetRecord>("SubnetRecord", &record_bytes)?;
            Ok::<_, RegistryFetchError>((subnet_principal, record))
        })
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await?;

    let node_principals = assigned_node_principals_from_subnets(&subnet_records)?;
    let node_records = stream::iter(node_principals.iter().cloned())
        .map(|node_principal| async move {
            let key = node_record_key(&node_principal);
            let record_bytes = get_registry_value(agent, registry_canister, &key, registry_version)
                .await
                .map_err(|error| translate_missing_node_record(&node_principal, error))?;
            let record = decode_message::<NodeRecord>("NodeRecord", &record_bytes)?;
            Ok::<_, RegistryFetchError>((node_principal, record))
        })
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await?;

    let node_operator_references = node_operator_references_from_records(&node_records)?;

    let node_operator_records = stream::iter(node_operator_references)
        .map(
            |(node_operator_principal, referencing_node_principals)| async move {
                let key = node_operator_record_key(&node_operator_principal);
                let record_bytes =
                    get_registry_value(agent, registry_canister, &key, registry_version)
                        .await
                        .map_err(|error| {
                            translate_missing_node_operator_record(
                                &node_operator_principal,
                                &referencing_node_principals,
                                error,
                            )
                        })?;
                let record =
                    decode_message::<NodeOperatorRecord>("NodeOperatorRecord", &record_bytes)?;
                Ok::<_, RegistryFetchError>((node_operator_principal, record))
            },
        )
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await?;

    #[cfg(feature = "nns-host")]
    let data_center_records = match scope {
        RegistryRelationInventoryScope::BaseRelations => BTreeMap::new(),
        RegistryRelationInventoryScope::WithDataCenters => {
            fetch_data_center_records_for_inventory(
                agent,
                registry_canister,
                registry_version,
                &node_operator_records,
            )
            .await?
        }
    };
    #[cfg(not(feature = "nns-host"))]
    let _ = scope;

    Ok(RegistryRelationInventory {
        node_principals,
        node_records,
        node_operator_records,
        subnet_records,
        #[cfg(feature = "nns-host")]
        data_center_records,
    })
}

fn translate_missing_node_record(
    node_principal: &str,
    error: RegistryFetchError,
) -> RegistryFetchError {
    if registry_key_not_present_for(&error, &node_record_key(node_principal)) {
        return RegistryFetchError::MissingNodeRecord {
            node_principal: node_principal.to_string(),
        };
    }
    error
}

fn translate_missing_node_operator_record(
    node_operator_principal: &str,
    referencing_node_principals: &[String],
    error: RegistryFetchError,
) -> RegistryFetchError {
    if registry_key_not_present_for(&error, &node_operator_record_key(node_operator_principal)) {
        return RegistryFetchError::MissingNodeOperatorRecord {
            node_operator_principal: node_operator_principal.to_string(),
            referencing_node_principals: referencing_node_principals.to_vec(),
        };
    }
    error
}

fn registry_key_not_present_for(error: &RegistryFetchError, expected_key: &str) -> bool {
    matches!(
        error,
        RegistryFetchError::RegistryValue { key, code, .. }
            if key == expected_key && code == "key_not_present"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_node_key_not_present_becomes_missing_node_relation() {
        let node_principal = "node-a";
        let error = RegistryFetchError::RegistryValue {
            key: node_record_key(node_principal),
            code: "key_not_present".to_string(),
            reason: "missing at requested version".to_string(),
        };

        let translated = translate_missing_node_record(node_principal, error);

        assert!(matches!(
            translated,
            RegistryFetchError::MissingNodeRecord { node_principal: found }
                if found == node_principal
        ));
    }

    #[test]
    fn required_operator_key_not_present_keeps_all_referencing_nodes() {
        let node_operator_principal = "operator-a";
        let referencing_node_principals = vec!["node-a".to_string(), "node-b".to_string()];
        let error = RegistryFetchError::RegistryValue {
            key: node_operator_record_key(node_operator_principal),
            code: "key_not_present".to_string(),
            reason: "missing at requested version".to_string(),
        };

        let translated = translate_missing_node_operator_record(
            node_operator_principal,
            &referencing_node_principals,
            error,
        );

        assert!(matches!(
            translated,
            RegistryFetchError::MissingNodeOperatorRecord {
                node_operator_principal: found_operator,
                referencing_node_principals: found_nodes,
            } if found_operator == node_operator_principal
                && found_nodes == referencing_node_principals
        ));
    }

    #[test]
    fn unrelated_registry_value_error_is_preserved() {
        let error = RegistryFetchError::RegistryValue {
            key: node_record_key("node-a"),
            code: "version_not_latest".to_string(),
            reason: "version was compacted".to_string(),
        };

        let translated = translate_missing_node_record("node-a", error);

        assert!(matches!(
            translated,
            RegistryFetchError::RegistryValue { key, code, reason }
                if key == node_record_key("node-a")
                    && code == "version_not_latest"
                    && reason == "version was compacted"
        ));
    }

    #[test]
    fn operator_reference_context_is_canonically_ordered() {
        let node_a = Principal::self_authenticating(b"node-a").to_text();
        let node_b = Principal::self_authenticating(b"node-b").to_text();
        let node_operator = Principal::self_authenticating(b"operator").to_text();
        let node_operator_id = Principal::from_text(&node_operator)
            .expect("operator principal")
            .as_slice()
            .to_vec();
        let records = BTreeMap::from([
            (
                node_b.clone(),
                NodeRecord {
                    node_operator_id: node_operator_id.clone(),
                },
            ),
            (node_a.clone(), NodeRecord { node_operator_id }),
        ]);

        let references =
            node_operator_references_from_records(&records).expect("operator references");

        assert_eq!(references.get(&node_operator), Some(&vec![node_a, node_b]));
    }
}
