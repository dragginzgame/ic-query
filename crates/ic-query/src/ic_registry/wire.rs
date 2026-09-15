use super::proto::LargeValueChunkKeys;
use crate::ic_registry::RegistryFetchError;
#[cfg(feature = "nns-host")]
use candid::Principal;
use candid::{CandidType, Deserialize};

///
/// RegistryValueContent
///
/// Decoded content returned by a registry value lookup.
///

#[derive(Debug)]
pub(super) enum RegistryValueContent {
    Value(Vec<u8>),
    LargeValueChunkKeys(LargeValueChunkKeys),
}

///
/// RegistryValueEncoding
///
/// Transport representation used to complete one Registry value.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RegistryValueEncoding {
    Inline,
    Chunked,
}

///
/// RegistryVersionedValue
///
/// Completed value plus the individual mutation provenance returned by Registry.
///

#[derive(Debug, Eq, PartialEq)]
pub(super) struct RegistryVersionedValue {
    pub(super) value: Vec<u8>,
    pub(super) version: u64,
    pub(super) timestamp_nanoseconds: u64,
    pub(super) encoding: RegistryValueEncoding,
}

///
/// RegistryVersionedValueFailure
///
/// Value-fetch failure retaining response provenance when Registry supplied it.
///

#[derive(Debug)]
pub(super) struct RegistryVersionedValueFailure {
    pub(super) source: RegistryFetchError,
    pub(super) returned_version: Option<u64>,
}

///
/// RegistryGetChunkRequest
///
/// Candid request for one chunk of a large registry value.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct RegistryGetChunkRequest {
    pub(super) content_sha256: Option<Vec<u8>>,
}

///
/// RegistryChunk
///
/// Candid response containing one validated registry value chunk.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct RegistryChunk {
    pub(super) content: Option<Vec<u8>>,
}

///
/// ListNodeProvidersResponse
///
/// Candid governance response containing the current node providers.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[cfg(feature = "nns-host")]
pub(super) struct ListNodeProvidersResponse {
    pub(super) node_providers: Vec<GovernanceNodeProvider>,
}

///
/// GovernanceNodeProvider
///
/// Candid node provider record returned by NNS governance.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[cfg(feature = "nns-host")]
pub(super) struct GovernanceNodeProvider {
    pub(super) id: Option<Principal>,
    pub(super) reward_account: Option<GovernanceAccountIdentifier>,
}

///
/// GovernanceAccountIdentifier
///
/// Candid account identifier attached to a governance node provider.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[cfg(feature = "nns-host")]
pub(super) struct GovernanceAccountIdentifier {
    pub(super) hash: Vec<u8>,
}
