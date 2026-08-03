use crate::nns::NnsInventoryCacheRequest;
#[cfg(feature = "host")]
use crate::nns::inventory::NnsInventoryListInput;
use crate::subnet_catalog::SubnetKind;

///
/// NnsNodeListRequest
///
/// Request for an optionally filtered NNS node inventory report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeListRequest {
    pub cache: NnsInventoryCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub filters: NnsNodeListFilters,
}

impl NnsNodeListRequest {
    #[must_use]
    pub fn new(
        cache: NnsInventoryCacheRequest,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            cache,
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            filters: NnsNodeListFilters::default(),
        }
    }

    #[must_use]
    pub fn with_filters(mut self, filters: NnsNodeListFilters) -> Self {
        self.filters = filters;
        self
    }

    #[must_use]
    pub fn with_subnet(mut self, subnet: impl Into<String>) -> Self {
        self.filters.subnet = Some(subnet.into());
        self
    }

    #[must_use]
    pub const fn with_subnet_kind(mut self, subnet_kind: SubnetKind) -> Self {
        self.filters.subnet_kind = Some(subnet_kind);
        self
    }

    #[must_use]
    pub fn with_data_center(mut self, data_center: impl Into<String>) -> Self {
        self.filters.data_center = Some(data_center.into());
        self
    }

    #[must_use]
    pub fn with_node_provider(mut self, node_provider: impl Into<String>) -> Self {
        self.filters.node_provider = Some(node_provider.into());
        self
    }

    #[must_use]
    pub fn with_node_operator(mut self, node_operator: impl Into<String>) -> Self {
        self.filters.node_operator = Some(node_operator.into());
        self
    }
}

#[cfg(feature = "host")]
impl NnsInventoryListInput for NnsNodeListRequest {
    fn cache(&self) -> &NnsInventoryCacheRequest {
        &self.cache
    }

    fn source_endpoint(&self) -> &str {
        &self.source_endpoint
    }

    fn now_unix_secs(&self) -> u64 {
        self.now_unix_secs
    }
}

///
/// NnsNodeListFilters
///
/// Relation filters applied to an NNS node inventory view.
///

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NnsNodeListFilters {
    pub subnet: Option<String>,
    pub subnet_kind: Option<SubnetKind>,
    pub data_center: Option<String>,
    pub node_provider: Option<String>,
    pub node_operator: Option<String>,
}

impl NnsNodeListFilters {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.subnet.is_none()
            && self.subnet_kind.is_none()
            && self.data_center.is_none()
            && self.node_provider.is_none()
            && self.node_operator.is_none()
    }
}
