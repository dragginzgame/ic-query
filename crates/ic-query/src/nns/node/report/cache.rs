use super::{
    NNS_NODE_CACHE_DIR, NNS_NODE_CACHE_FILE, NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
    NnsNodeCacheRequest, NnsNodeHostError, NnsNodeListReport,
};

nns_leaf_cache!(
    nns_node_cache_path,
    nns_node_refresh_lock_path,
    load_cached_nns_node_report,
    NnsNodeCacheRequest,
    NnsNodeListReport,
    NnsNodeHostError,
    NNS_NODE_CACHE_DIR,
    NNS_NODE_CACHE_FILE,
    NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
);
