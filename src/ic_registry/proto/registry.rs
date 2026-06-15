use prost::Oneof;

///
/// RegistryError
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryError {
    #[prost(enumeration = "RegistryErrorCode", tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub reason: String,
    #[prost(bytes = "vec", tag = "3")]
    pub key: Vec<u8>,
}

///
/// RegistryErrorCode
///
#[derive(Clone, Copy, Debug, prost::Enumeration, Eq, PartialEq)]
#[repr(i32)]
pub enum RegistryErrorCode {
    MalformedMessage = 0,
    KeyNotPresent = 1,
    KeyAlreadyPresent = 2,
    VersionNotLatest = 3,
    VersionBeyondLatest = 4,
    Authorization = 5,
    InternalError = 999,
}

///
/// LargeValueChunkKeys
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct LargeValueChunkKeys {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub chunk_content_sha256s: Vec<Vec<u8>>,
}

///
/// RegistryGetValueRequest
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryGetValueRequest {
    #[prost(message, optional, tag = "1")]
    pub version: Option<UInt64Value>,
    #[prost(bytes = "vec", tag = "2")]
    pub key: Vec<u8>,
}

///
/// UInt64Value
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct UInt64Value {
    #[prost(uint64, tag = "1")]
    pub value: u64,
}

///
/// RegistryGetValueResponse
///
/// This is the high-capacity `get_value` response wire shape. The upstream
/// proto calls it `HighCapacityRegistryGetValueResponse`; this crate keeps the
/// local name narrow because this adapter never exposes protobuf types.
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryGetValueResponse {
    #[prost(message, optional, tag = "1")]
    pub error: Option<RegistryError>,
    #[prost(uint64, tag = "2")]
    pub version: u64,
    #[prost(oneof = "registry_get_value_response::Content", tags = "3, 4")]
    pub content: Option<registry_get_value_response::Content>,
    #[prost(uint64, tag = "5")]
    pub timestamp_nanoseconds: u64,
}

pub mod registry_get_value_response {
    use super::{LargeValueChunkKeys, Oneof};

    ///
    /// Content
    ///
    #[derive(Clone, Eq, Oneof, PartialEq)]
    pub enum Content {
        #[prost(bytes = "vec", tag = "3")]
        Value(Vec<u8>),
        #[prost(message, tag = "4")]
        LargeValueChunkKeys(LargeValueChunkKeys),
    }
}

///
/// RegistryGetLatestVersionResponse
///
#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryGetLatestVersionResponse {
    #[prost(uint64, tag = "1")]
    pub version: u64,
}
