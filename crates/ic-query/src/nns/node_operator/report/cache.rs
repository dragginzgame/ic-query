use super::{
    NNS_NODE_OPERATOR_CACHE_DIR, NNS_NODE_OPERATOR_CACHE_FILE,
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION, NnsNodeOperatorCacheRequest,
    NnsNodeOperatorHostError, NnsNodeOperatorListReport,
};

nns_leaf_cache!(
    nns_node_operator_cache_path,
    nns_node_operator_refresh_lock_path,
    load_cached_nns_node_operator_report,
    NnsNodeOperatorCacheRequest,
    NnsNodeOperatorListReport,
    NnsNodeOperatorHostError,
    NNS_NODE_OPERATOR_CACHE_DIR,
    NNS_NODE_OPERATOR_CACHE_FILE,
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
);
