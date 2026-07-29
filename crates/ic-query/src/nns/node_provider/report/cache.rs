use super::{
    NNS_NODE_PROVIDER_CACHE_DIR, NNS_NODE_PROVIDER_CACHE_FILE,
    NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION, NnsNodeProviderCacheRequest,
    NnsNodeProviderHostError, NnsNodeProviderListReport,
};

nns_leaf_cache!(
    nns_node_provider_cache_path,
    nns_node_provider_refresh_lock_path,
    load_cached_nns_node_provider_report,
    NnsNodeProviderCacheRequest,
    NnsNodeProviderListReport,
    NnsNodeProviderHostError,
    NNS_NODE_PROVIDER_CACHE_DIR,
    NNS_NODE_PROVIDER_CACHE_FILE,
    NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
);
