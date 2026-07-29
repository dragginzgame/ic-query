#[cfg(feature = "host")]
mod cache;
#[cfg(feature = "host")]
mod model;
#[cfg(feature = "host")]
mod paths;

#[cfg(feature = "host")]
pub(in crate::nns) use cache::{load_nns_leaf_json_cache, write_nns_leaf_json_refresh_cache};
#[cfg(feature = "host")]
pub(super) use model::{NnsLeafCacheRequest, NnsLeafRefreshRequest};
#[cfg(feature = "host")]
pub(in crate::nns) use paths::NnsLeafCachePaths;

#[cfg(all(test, feature = "host"))]
mod tests;
