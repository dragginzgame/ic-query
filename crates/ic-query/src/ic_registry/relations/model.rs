use crate::ic_registry::proto::{DataCenterRecord, NodeOperatorRecord, NodeRecord, SubnetRecord};
use std::collections::{BTreeMap, BTreeSet};

///
/// RegistryRelationInventory
///
/// Registry records required to join nodes with operators, subnets, and data centers.
///

pub(in crate::ic_registry) struct RegistryRelationInventory {
    pub(in crate::ic_registry) node_principals: BTreeSet<String>,
    pub(in crate::ic_registry) node_records: BTreeMap<String, NodeRecord>,
    pub(in crate::ic_registry) node_operator_records: BTreeMap<String, NodeOperatorRecord>,
    pub(in crate::ic_registry) subnet_records: BTreeMap<String, SubnetRecord>,
    pub(in crate::ic_registry) data_center_records: BTreeMap<String, DataCenterRecord>,
}

///
/// RegistryRelationInventoryScope
///
/// Registry inventory rows collected for relation and topology joins.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::ic_registry) enum RegistryRelationInventoryScope {
    BaseRelations,
    WithDataCenters,
}
