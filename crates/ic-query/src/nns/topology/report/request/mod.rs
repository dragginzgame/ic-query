#[cfg(feature = "host")]
mod cache;
#[cfg(feature = "host")]
mod list;
mod model;
#[cfg(feature = "host")]
mod refresh;

#[cfg(feature = "host")]
pub(super) use list::{inventory_list_request, node_list_request, subnet_catalog_list_request};
pub use model::{NnsTopologyReadRequest, NnsTopologyRefreshRequest};
#[cfg(feature = "host")]
pub(super) use model::{TopologyRefreshParts, TopologyRequestParts, summary_request_from};
#[cfg(feature = "host")]
pub(super) use refresh::{inventory_refresh_request, subnet_catalog_refresh_request};
