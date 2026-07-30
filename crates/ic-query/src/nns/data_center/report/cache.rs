use super::{
    NNS_DATA_CENTER_CACHE_DIR, NNS_DATA_CENTER_CACHE_FILE,
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NnsDataCenterHostError, NnsDataCenterListReport,
    NnsInventoryCacheRequest,
};

nns_leaf_cache!(
    nns_data_center_cache_path,
    nns_data_center_refresh_lock_path,
    load_cached_nns_data_center_report,
    NnsInventoryCacheRequest,
    NnsDataCenterListReport,
    NnsDataCenterHostError,
    NNS_DATA_CENTER_CACHE_DIR,
    NNS_DATA_CENTER_CACHE_FILE,
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
);
