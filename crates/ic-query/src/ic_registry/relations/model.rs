#[cfg(feature = "nns-host")]
use ic_host_tools::registry::proto::DataCenterRecord;
use ic_host_tools::registry::proto::{NodeOperatorRecord, SubnetRecord};
use std::collections::{BTreeMap, BTreeSet};

///
/// RegistryRelationInventory
///
/// Registry records required to join nodes with operators, subnets, and data centers.
///

pub(in crate::ic_registry) struct RegistryRelationInventory {
    pub(in crate::ic_registry) node_principals: BTreeSet<String>,
    /// Canonical operator IDs admitted from the fetched node records.
    pub(in crate::ic_registry) node_operator_principals: BTreeMap<String, String>,
    pub(in crate::ic_registry) node_operator_records: BTreeMap<String, NodeOperatorRecord>,
    pub(in crate::ic_registry) subnet_records: BTreeMap<String, SubnetRecord>,
    #[cfg(feature = "nns-host")]
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
    #[cfg(feature = "nns-host")]
    WithDataCenters,
}
