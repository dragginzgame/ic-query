use super::proto::LargeValueChunkKeys;
use candid::{CandidType, Deserialize, Principal};

///
/// RegistryValueContent
///
#[derive(Debug)]
pub(super) enum RegistryValueContent {
    Value(Vec<u8>),
    LargeValueChunkKeys(LargeValueChunkKeys),
}

///
/// RegistryGetChunkRequest
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct RegistryGetChunkRequest {
    pub(super) content_sha256: Option<Vec<u8>>,
}

///
/// RegistryChunk
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct RegistryChunk {
    pub(super) content: Option<Vec<u8>>,
}

///
/// ListNodeProvidersResponse
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ListNodeProvidersResponse {
    pub(super) node_providers: Vec<GovernanceNodeProvider>,
}

///
/// GovernanceNodeProvider
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct GovernanceNodeProvider {
    pub(super) id: Option<Principal>,
    pub(super) reward_account: Option<GovernanceAccountIdentifier>,
}

///
/// GovernanceAccountIdentifier
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct GovernanceAccountIdentifier {
    pub(super) hash: Vec<u8>,
}
