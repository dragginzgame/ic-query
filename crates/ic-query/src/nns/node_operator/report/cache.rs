use super::{
    NNS_NODE_OPERATOR_CACHE_DIR, NNS_NODE_OPERATOR_CACHE_FILE,
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION, NnsInventoryCacheRequest,
    NnsNodeOperatorListReport,
};

nns_leaf_cache!(
    nns_node_operator_cache_path,
    nns_node_operator_refresh_lock_path,
    load_cached_nns_node_operator_report,
    NnsInventoryCacheRequest,
    NnsNodeOperatorListReport,
    NNS_NODE_OPERATOR_CACHE_DIR,
    NNS_NODE_OPERATOR_CACHE_FILE,
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
);
