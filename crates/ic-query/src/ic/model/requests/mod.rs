//! Module: ic::model::requests
//!
//! Responsibility: expose public IC Dashboard request, query, filter, and selector contracts.
//! Does not own: reports, host source data, errors, transport, or projection.
//! Boundary: preserves one explicit request facade across bounded Dashboard capability families.

mod canisters;
mod metrics;
mod network;

pub use canisters::{
    IcCanisterCountRequest, IcCanisterFilters, IcCanisterPageRequest, IcCanisterRequest,
};
pub use metrics::{IcMetricKind, IcMetricQuery, IcMetricRequest};
pub use network::{IcBoundaryNodeDataCentersRequest, IcDailyStatsQuery, IcDailyStatsRequest};
