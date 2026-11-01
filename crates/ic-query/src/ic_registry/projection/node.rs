use crate::ic_registry::{
    MainnetNodeList, MainnetRegistryFetchRequest, RegistryFetchError,
    projection::subnet_kind_from_registry,
    relations::{
        RegistryRelationInventory, ResolvedNodeRelation, node_subnet_assignments_from_records,
        resolved_node_relations,
    },
};
use crate::nns::node::NnsNodeRow;
use ic_host_tools::registry::proto::SubnetRecord;
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn node_list_from_inventory(
    request: &MainnetRegistryFetchRequest,
    inventory: RegistryRelationInventory,
    registry_version: u64,
) -> Result<MainnetNodeList, RegistryFetchError> {
    let node_subnets = node_subnet_assignments_from_records(&inventory.subnet_records)?;
    let node_relations = resolved_node_relations(
        &inventory.node_principals,
        &inventory.node_operator_principals,
        &inventory.node_operator_records,
    )?;
    let nodes = node_relations
        .into_iter()
        .map(|(principal, relation)| {
            node_from_relation(
                principal,
                relation,
                &inventory.subnet_records,
                &node_subnets,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MainnetNodeList {
        provenance: request.inventory_provenance(registry_version),
        nodes,
    })
}

fn node_from_relation(
    principal: String,
    relation: ResolvedNodeRelation,
    subnet_records: &BTreeMap<String, SubnetRecord>,
    node_subnets: &BTreeMap<String, String>,
) -> Result<NnsNodeRow, RegistryFetchError> {
    let subnet_principal =
        node_subnets
            .get(&principal)
            .ok_or(RegistryFetchError::MissingField {
                field: "node_subnet_assignment",
            })?;
    let subnet_record =
        subnet_records
            .get(subnet_principal)
            .ok_or(RegistryFetchError::MissingField {
                field: "subnet_record",
            })?;
    Ok(NnsNodeRow {
        node_principal: principal,
        node_operator_principal: relation.node_operator_principal,
        node_provider_principal: relation.node_provider_principal,
        subnet_principal: subnet_principal.clone(),
        subnet_kind: subnet_kind_from_registry(subnet_record.subnet_type),
        data_center_id: relation.data_center_id,
    })
}
