///
/// PrincipalId
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct PrincipalId {
    #[prost(bytes = "vec", tag = "1")]
    pub raw: Vec<u8>,
}

///
/// CanisterId
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct CanisterId {
    #[prost(message, optional, tag = "1")]
    pub principal_id: Option<PrincipalId>,
}

///
/// SubnetId
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct SubnetId {
    #[prost(message, optional, tag = "1")]
    pub principal_id: Option<PrincipalId>,
}
