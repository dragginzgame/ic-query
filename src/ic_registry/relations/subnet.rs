use crate::ic_registry::{RegistryFetchError, principal_text_from_raw, proto::SubnetRecord};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::ic_registry) fn assigned_node_principals_from_subnets(
    subnet_records: &BTreeMap<String, SubnetRecord>,
) -> Result<BTreeSet<String>, RegistryFetchError> {
    let mut node_principals = BTreeSet::new();
    for record in subnet_records.values() {
        for raw in &record.membership {
            node_principals.insert(principal_text_from_raw(raw, "subnet_record.membership")?);
        }
    }
    Ok(node_principals)
}

pub(in crate::ic_registry) fn node_subnet_assignments_from_records(
    subnet_records: &BTreeMap<String, SubnetRecord>,
) -> Result<BTreeMap<String, String>, RegistryFetchError> {
    let mut assignments = BTreeMap::new();
    for (subnet_principal, record) in subnet_records {
        for raw in &record.membership {
            let node_principal = principal_text_from_raw(raw, "subnet_record.membership")?;
            assignments.insert(node_principal, subnet_principal.clone());
        }
    }
    Ok(assignments)
}
