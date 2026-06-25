///
/// NodeRecord
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct NodeRecord {
    #[prost(bytes = "vec", tag = "15")]
    pub node_operator_id: Vec<u8>,
}

///
/// NodeOperatorRecord
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct NodeOperatorRecord {
    #[prost(bytes = "vec", tag = "1")]
    pub node_operator_principal_id: Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub node_allowance: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub node_provider_principal_id: Vec<u8>,
    #[prost(string, tag = "4")]
    pub dc_id: String,
}

///
/// DataCenterRecord
///
#[derive(Clone, prost::Message, PartialEq)]
pub struct DataCenterRecord {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub region: String,
    #[prost(string, tag = "3")]
    pub owner: String,
    #[prost(message, optional, tag = "4")]
    pub gps: Option<Gps>,
}

///
/// Gps
///
#[derive(Clone, prost::Message, PartialEq)]
pub struct Gps {
    #[prost(float, tag = "1")]
    pub latitude: f32,
    #[prost(float, tag = "2")]
    pub longitude: f32,
}
