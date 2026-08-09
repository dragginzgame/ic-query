//! Module: ic::model::requests
//!
//! Responsibility: expose public IC Dashboard request, query, filter, and selector contracts.
//! Does not own: reports, host source data, errors, transport, or projection.
//! Boundary: preserves one explicit request facade across bounded Dashboard capability families.

mod canisters;
mod icrc_analytics;
mod icrc_index;
mod metrics;
mod network;
mod node_provider_rewards;
mod replica_versions;

pub use canisters::{
    IcCanisterCountRequest, IcCanisterFilters, IcCanisterPageRequest, IcCanisterRequest,
};
pub use icrc_analytics::{
    IcIcrcAnalyticsRequest, IcIcrcIndexedCountKind, IcIcrcIndexedCountRequest,
    IcIcrcTokenValueQuery, IcIcrcTokenValueRequest, IcIcrcTotalSupplyQuery,
    IcIcrcTotalSupplyRequest,
};
pub use icrc_index::{
    IcIcrcAccountInfoRequest, IcIcrcAccountListQuery, IcIcrcAccountListRequest, IcIcrcAccountSort,
    IcIcrcHolderListQuery, IcIcrcHolderListRequest, IcIcrcHolderSort,
};
pub use metrics::{IcMetricKind, IcMetricQuery, IcMetricRequest};
pub use network::{IcBoundaryNodeDataCentersRequest, IcDailyStatsQuery, IcDailyStatsRequest};
pub use node_provider_rewards::{
    IcNodeProviderRewardHistoryQuery, IcNodeProviderRewardHistoryRequest,
    IcNodeProviderRewardInfoRequest, IcNodeProviderRewardListQuery,
    IcNodeProviderRewardListRequest,
};
pub use replica_versions::{
    IcReplicaVersionInfoRequest, IcReplicaVersionListQuery, IcReplicaVersionListRequest,
};
