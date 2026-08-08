//! Module: ic::api_boundary_node
//!
//! Responsibility: certified IC state-tree API boundary-node reporting.
//! Does not own: Dashboard location aggregates, operational health, or command parsing.
//! Boundary: exposes one authenticated complete collection without Registry or Dashboard joins.

#[cfg(feature = "ic-state-host")]
mod host;
mod model;
mod text;
mod validation;

#[cfg(feature = "ic-state-host")]
pub use host::{
    IcApiBoundaryNodeSource, LiveIcStateSource, build_ic_api_boundary_node_report,
    build_ic_api_boundary_node_report_with_source,
};
#[cfg(feature = "ic-state-host")]
pub use model::{
    IcApiBoundaryNodeHostError, IcApiBoundaryNodeSourceData, IcApiBoundaryNodeSourceRequest,
};
pub use model::{
    IcApiBoundaryNodeReport, IcApiBoundaryNodeRequest, IcApiBoundaryNodeRow,
    IcCertifiedStateProvenance,
};
pub use text::ic_api_boundary_node_report_text;
#[cfg(feature = "ic-state-host")]
pub(super) use validation::report_from_source;

/// Current schema version for certified API boundary-node reports.
pub const IC_API_BOUNDARY_NODE_REPORT_SCHEMA_VERSION: u32 = 1;

/// Default mainnet IC API endpoint for certified state-tree reports.
pub const DEFAULT_IC_STATE_SOURCE_ENDPOINT: &str = "https://icp-api.io";

/// Maximum API boundary-node rows accepted from one certified state tree.
pub const MAX_IC_API_BOUNDARY_NODE_ROWS: usize = 1_000;

#[cfg(feature = "ic-state-host")]
pub(super) const IC_API_BOUNDARY_NODE_AUTHORITY: &str = "certified_ic_state_tree";

#[cfg(test)]
mod tests;
