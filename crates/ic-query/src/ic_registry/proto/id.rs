///
/// PrincipalId
///
/// Protobuf principal identifier bytes used by registry records.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct PrincipalId {
    #[prost(bytes = "vec", tag = "1")]
    pub raw: Vec<u8>,
}

///
/// CanisterId
///
/// Protobuf canister identifier wrapper used by registry records.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct CanisterId {
    #[prost(message, optional, tag = "1")]
    pub principal_id: Option<PrincipalId>,
}

///
/// SubnetId
///
/// Protobuf subnet identifier wrapper used by registry records.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct SubnetId {
    #[prost(message, optional, tag = "1")]
    pub principal_id: Option<PrincipalId>,
}
