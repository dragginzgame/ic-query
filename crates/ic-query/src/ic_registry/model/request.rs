use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;

///
/// MainnetRegistryFetchRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetRegistryFetchRequest {
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl MainnetRegistryFetchRequest {
    #[must_use]
    pub fn new(fetched_at: String) -> Self {
        Self {
            endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            fetched_at,
            fetched_by: "ic-query".to_string(),
        }
    }
}
