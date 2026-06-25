use super::{CanisterId, SubnetId};

///
/// CanisterIdRange
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct CanisterIdRange {
    #[prost(message, optional, tag = "3")]
    pub start_canister_id: Option<CanisterId>,
    #[prost(message, optional, tag = "4")]
    pub end_canister_id: Option<CanisterId>,
}

///
/// RoutingTable
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RoutingTable {
    #[prost(message, repeated, tag = "1")]
    pub entries: Vec<RoutingTableEntry>,
}

///
/// RoutingTableEntry
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RoutingTableEntry {
    #[prost(message, optional, tag = "1")]
    pub range: Option<CanisterIdRange>,
    #[prost(message, optional, tag = "2")]
    pub subnet_id: Option<SubnetId>,
}
