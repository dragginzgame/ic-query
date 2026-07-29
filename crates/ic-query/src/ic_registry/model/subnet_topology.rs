use crate::subnet_catalog::SubnetKind;

///
/// MainnetSubnetTopology
///
/// Exact-version mainnet Registry projection of Subnets and their node providers.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSubnetTopology {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub subnets: Vec<MainnetSubnetTopologySubnet>,
}

///
/// MainnetSubnetTopologySubnet
///
/// One Registry Subnet with raw execution kind and provider membership counts.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSubnetTopologySubnet {
    pub subnet_principal: String,
    pub subnet_kind: SubnetKind,
    pub node_count: u32,
    pub node_providers: Vec<MainnetSubnetTopologyNodeProvider>,
}

///
/// MainnetSubnetTopologyNodeProvider
///
/// Registry-derived node count for one provider on one Subnet.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSubnetTopologyNodeProvider {
    pub node_provider_principal: String,
    pub node_count: u32,
}
