use prost::Oneof;

///
/// RegistryError
///
/// Protobuf error returned by a registry canister lookup.
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
/// Protobuf error category returned by the registry canister.
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
/// Protobuf chunk keys used to retrieve a large registry value.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct LargeValueChunkKeys {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub chunk_content_sha256s: Vec<Vec<u8>>,
}

///
/// RegistryGetValueRequest
///
/// Protobuf request for a registry key at an optional version.
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
/// Protobuf wrapper for an unsigned 64-bit registry value.
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
///

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
/// Protobuf response containing the latest registry version.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
pub struct RegistryGetLatestVersionResponse {
    #[prost(uint64, tag = "1")]
    pub version: u64,
}

///
/// RegistryGetChangesSinceRequest
///
/// Protobuf request for one certified Registry delta batch after a version.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
#[cfg(feature = "nns-host")]
pub struct RegistryGetChangesSinceRequest {
    /// Last Registry version already held by the caller.
    #[prost(uint64, tag = "1")]
    pub version: u64,
}

///
/// HighCapacityRegistryAtomicMutateRequest
///
/// One atomic Registry delta committed under a certified version label.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
#[cfg(feature = "nns-host")]
pub struct HighCapacityRegistryAtomicMutateRequest {
    /// Ordered mutations applied by the Registry version.
    #[prost(message, repeated, tag = "1")]
    pub mutations: Vec<HighCapacityRegistryMutation>,
    /// Preconditions checked before applying the atomic mutation.
    #[prost(message, repeated, tag = "5")]
    pub preconditions: Vec<RegistryPrecondition>,
    /// Registry-assigned mutation timestamp in nanoseconds since the Unix epoch.
    #[prost(uint64, tag = "6")]
    pub timestamp_nanoseconds: u64,
}

///
/// HighCapacityRegistryMutation
///
/// One mutation in a certified Registry atomic mutation request.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
#[cfg(feature = "nns-host")]
pub struct HighCapacityRegistryMutation {
    /// Raw `RegistryMutation.Type` numeric discriminant.
    #[prost(int32, tag = "1")]
    pub mutation_type: i32,
    /// Raw Registry key bytes.
    #[prost(bytes = "vec", tag = "2")]
    pub key: Vec<u8>,
    /// Inline value bytes or large-value chunk references.
    #[prost(oneof = "high_capacity_registry_mutation::Content", tags = "3, 4")]
    pub content: Option<high_capacity_registry_mutation::Content>,
}

#[cfg(feature = "nns-host")]
pub mod high_capacity_registry_mutation {
    use super::LargeValueChunkKeys;
    use prost::Oneof;

    ///
    /// Content
    ///
    /// Value representation in one high-capacity Registry mutation.
    ///

    #[derive(Clone, Eq, Oneof, PartialEq)]
    pub enum Content {
        /// Complete inline value bytes.
        #[prost(bytes = "vec", tag = "3")]
        Value(Vec<u8>),
        /// Content-addressed chunks that require bounded follow-up retrieval.
        #[prost(message, tag = "4")]
        LargeValueChunkKeys(LargeValueChunkKeys),
    }
}

///
/// RegistryMutationType
///
/// Official Registry mutation discriminants used by certified deltas.
///

#[derive(Clone, Copy, Debug, prost::Enumeration, Eq, PartialEq)]
#[cfg(feature = "nns-host")]
#[repr(i32)]
pub enum RegistryMutationType {
    Insert = 0,
    Update = 1,
    Delete = 2,
    Upsert = 4,
}

///
/// RegistryPrecondition
///
/// Expected key version attached to one atomic Registry mutation.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
#[cfg(feature = "nns-host")]
pub struct RegistryPrecondition {
    /// Registry key whose version is constrained.
    #[prost(bytes = "vec", tag = "1")]
    pub key: Vec<u8>,
    /// Required Registry version for the key.
    #[prost(uint64, tag = "2")]
    pub expected_version: u64,
}

///
/// RegistryCertifiedResponse
///
/// Protobuf response returned by the Registry's certified query methods.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
#[cfg(feature = "nns-host")]
pub struct RegistryCertifiedResponse {
    /// Certified mixed hash-tree witness.
    #[prost(message, optional, tag = "1")]
    pub hash_tree: Option<RegistryMixedHashTree>,
    /// CBOR system certificate authenticating the tree root.
    #[prost(bytes = "vec", tag = "2")]
    pub certificate: Vec<u8>,
}

///
/// RegistryMixedHashTree
///
/// Protobuf representation of a certified Registry mixed hash tree.
///

#[derive(Clone, Eq, prost::Message, PartialEq)]
#[cfg(feature = "nns-host")]
pub struct RegistryMixedHashTree {
    /// Encoded tree node.
    #[prost(oneof = "registry_mixed_hash_tree::Tree", tags = "1, 2, 3, 4, 5")]
    pub tree: Option<registry_mixed_hash_tree::Tree>,
}

#[cfg(feature = "nns-host")]
pub mod registry_mixed_hash_tree {
    use super::RegistryMixedHashTree;
    use prost::Oneof;

    ///
    /// Fork
    ///
    /// Two child branches in a Registry mixed hash tree.
    ///

    #[derive(Clone, Eq, prost::Message, PartialEq)]
    pub struct Fork {
        /// Left child branch.
        #[prost(message, optional, boxed, tag = "1")]
        pub left_tree: Option<Box<RegistryMixedHashTree>>,
        /// Right child branch.
        #[prost(message, optional, boxed, tag = "2")]
        pub right_tree: Option<Box<RegistryMixedHashTree>>,
    }

    ///
    /// Labeled
    ///
    /// Label and child branch in a Registry mixed hash tree.
    ///

    #[derive(Clone, Eq, prost::Message, PartialEq)]
    pub struct Labeled {
        /// Raw branch label.
        #[prost(bytes = "vec", tag = "1")]
        pub label: Vec<u8>,
        /// Labeled child branch.
        #[prost(message, optional, boxed, tag = "2")]
        pub subtree: Option<Box<RegistryMixedHashTree>>,
    }

    ///
    /// Tree
    ///
    /// One node in a Registry mixed hash tree.
    ///

    #[derive(Clone, Eq, Oneof, PartialEq)]
    pub enum Tree {
        /// Empty tree node.
        #[prost(message, tag = "1")]
        Empty(()),
        /// Fork containing two child branches.
        #[prost(message, tag = "2")]
        Fork(Box<Fork>),
        /// Labeled child branch.
        #[prost(message, tag = "3")]
        Labeled(Box<Labeled>),
        /// Visible leaf bytes.
        #[prost(bytes, tag = "4")]
        LeafData(Vec<u8>),
        /// Pruned SHA-256 subtree digest.
        #[prost(bytes, tag = "5")]
        PrunedDigest(Vec<u8>),
    }
}
