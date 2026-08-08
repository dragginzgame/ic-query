//! Module: cloud_engine::node
//!
//! Responsibility: official Dashboard CloudEngine Type4 node requests, reports, and text.
//! Does not own: native control-plane queries, Registry evidence, CLI parsing, or caching.
//! Boundary: selects the explicit Type4 node scope and preserves raw off-chain observations.

#[cfg(feature = "dashboard-host")]
mod host;
mod model;
mod text;

#[cfg(feature = "dashboard-host")]
pub use host::{
    CloudEngineNodeSource, build_cloud_engine_node_info_report,
    build_cloud_engine_node_info_report_with_source, build_cloud_engine_node_list_report,
    build_cloud_engine_node_list_report_with_source,
};
pub use model::{
    CloudEngineNodeInfoReport, CloudEngineNodeInfoRequest, CloudEngineNodeListReport,
    CloudEngineNodeListRequest, CloudEngineNodeRow,
};
#[cfg(feature = "dashboard-host")]
pub use model::{CloudEngineNodeInfoSourceData, CloudEngineNodeListSourceData};
pub use text::{cloud_engine_node_info_report_text, cloud_engine_node_list_report_text};

/// Dashboard reward-type filter selecting CloudEngine nodes.
pub const CLOUD_ENGINE_NODE_REWARD_TYPE: &str = "Type4";

/// Complete current Dashboard status scope requested by CloudEngine node lists.
pub const CLOUD_ENGINE_NODE_INCLUDED_STATUSES: [&str; 4] = ["DOWN", "UP", "DISABLED", "DEGRADED"];

/// Maximum Type4 rows accepted from one Dashboard node response.
pub const MAX_CLOUD_ENGINE_NODE_ROWS: u32 = 10_000;

#[cfg(test)]
mod tests;
