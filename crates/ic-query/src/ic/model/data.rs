//! Module: ic::model::data
//!
//! Responsibility: host-only Dashboard source request and returned-data contracts.
//! Does not own: public reports, caller requests, errors, transport, or validation.
//! Boundary: preserves source inputs and raw returned evidence for validation.

use super::{
    reports::{
        IcBoundaryNodeDataCenterRow, IcCanisterPageRow, IcCanisterUpgrade, IcDailyStatsRow,
        IcIcrcTotalSupplyObservation, IcMetricSeries, IcNodeProviderRewardHistoryObservation,
        IcNodeProviderRewardRow, IcReplicaVersionListRow, IcReplicaVersionSubnetRollout,
    },
    requests::{
        IcCanisterFilters, IcDailyStatsQuery, IcIcrcIndexedCountKind, IcIcrcTokenValueQuery,
        IcIcrcTotalSupplyQuery, IcMetricQuery, IcNodeProviderRewardHistoryQuery,
        IcNodeProviderRewardListQuery, IcReplicaVersionListQuery,
    },
};

///
/// IcSourceRequest
///
/// Shared endpoint and collection provenance for IC Dashboard source calls and results.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcSourceRequest {
    /// Dashboard API base endpoint.
    pub endpoint: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// Collector identity recorded in report provenance.
    pub fetched_by: String,
}

impl IcSourceRequest {
    /// Construct source-call provenance.
    #[must_use]
    pub fn new(
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

///
/// IcCanisterSourceData
///
/// Raw canister metadata and provenance returned by an IC Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Canister principal returned by the Dashboard.
    pub canister_id: String,
    /// Dashboard database row identifier.
    pub dashboard_id: u64,
    /// Raw optional Dashboard canister classification.
    pub canister_type: Option<String>,
    /// Raw Dashboard canister name.
    pub name: String,
    /// Subnet principal returned by the Dashboard.
    pub subnet_id: String,
    /// Controller principals returned by the Dashboard.
    pub controllers: Vec<String>,
    /// Raw Dashboard language label.
    pub language: String,
    /// Raw current module hash.
    pub module_hash: String,
    /// Raw Dashboard row update timestamp.
    pub dashboard_updated_at: String,
    /// Proposal-linked upgrades, or `None` when the Dashboard returned `null`.
    pub upgrades: Option<Vec<IcCanisterUpgrade>>,
}

///
/// IcCanisterCountSourceData
///
/// Raw filtered count and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterCountSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Filters applied by the source.
    pub filters: IcCanisterFilters,
    /// Number of matching Dashboard canister records.
    pub total: u64,
}

///
/// IcCanisterPageSourceData
///
/// Raw bounded canister page and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterPageSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Filters applied by the source.
    pub filters: IcCanisterFilters,
    /// Maximum rows requested from the source.
    pub requested_limit: u16,
    /// Exclusive forward cursor supplied to the source.
    pub after: Option<String>,
    /// Exclusive backward cursor supplied to the source.
    pub before: Option<String>,
    /// Cursor for an explicit request for the preceding page.
    pub previous_cursor: Option<String>,
    /// Cursor for an explicit request for the following page.
    pub next_cursor: Option<String>,
    /// Canister discovery rows returned by the source.
    pub rows: Vec<IcCanisterPageRow>,
}

///
/// IcMetricSourceData
///
/// Raw bounded metric series and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcMetricSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Metric query applied by the source.
    pub query: IcMetricQuery,
    /// Raw named time series returned by the source.
    pub series: Vec<IcMetricSeries>,
}

///
/// IcIcrcTotalSupplySourceData
///
/// Raw bounded ICRC total-supply series and provenance returned by a source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcTotalSupplySourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Canonical ledger canister principal queried by the source.
    pub ledger_canister_id: String,
    /// Total-supply query applied by the source.
    pub query: IcIcrcTotalSupplyQuery,
    /// Raw observations returned by the source.
    pub observations: Vec<IcIcrcTotalSupplyObservation>,
}

///
/// IcIcrcIndexedCountSourceData
///
/// Raw scalar count and provenance returned by an official ICRC analytics source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcIndexedCountSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Canonical ledger canister principal queried by the source.
    pub ledger_canister_id: String,
    /// Indexed resource counted by the source.
    pub kind: IcIcrcIndexedCountKind,
    /// Number of matching resources reported by the source.
    pub total: u64,
}

///
/// IcIcrcTokenValueSourceRow
///
/// One untrusted raw token-value record returned by an analytics source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcTokenValueSourceRow {
    /// Raw legacy price field, when returned.
    pub price: Option<String>,
    /// Raw legacy 24-hour volume field, when returned.
    pub volume_24h: Option<String>,
    /// Raw explicit price-in-USD field, when returned.
    pub price_usd: Option<String>,
    /// Raw explicit 24-hour volume-in-USD field, when returned.
    pub volume_24h_usd: Option<String>,
    /// External value provider, when returned.
    pub source: Option<String>,
    /// External value-provider URL, when returned.
    pub source_url: Option<String>,
    /// Raw optional observation timestamp returned by the source.
    pub timestamp_unix_secs: Option<u64>,
}

///
/// IcIcrcTokenValueSourceData
///
/// Raw bounded token-value series and provenance returned by a source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcTokenValueSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Canonical ledger canister principal queried by the source.
    pub ledger_canister_id: String,
    /// Token-value query applied by the source.
    pub query: IcIcrcTokenValueQuery,
    /// Untrusted raw rows returned by the source.
    pub rows: Vec<IcIcrcTokenValueSourceRow>,
}

///
/// IcDailyStatsSourceData
///
/// Raw bounded daily network-activity rows and provenance returned by a source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcDailyStatsSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Daily-statistics query applied by the source.
    pub query: IcDailyStatsQuery,
    /// Selected raw daily rows returned by the source.
    pub rows: Vec<IcDailyStatsRow>,
}

///
/// IcBoundaryNodeDataCentersSourceData
///
/// Raw boundary-node data-center rows and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcBoundaryNodeDataCentersSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Raw data-center rows returned by the source.
    pub rows: Vec<IcBoundaryNodeDataCenterRow>,
}

///
/// IcNodeProviderRewardListSourceData
///
/// Raw bounded node-provider reward page and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeProviderRewardListSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Page query applied by the source.
    pub query: IcNodeProviderRewardListQuery,
    /// Reward-index ceiling selected by the source.
    pub resolved_max_reward_index: u64,
    /// Number of reward records matching the selected ceiling.
    pub total_reward_records: u64,
    /// Raw reward rows returned by the source.
    pub rows: Vec<IcNodeProviderRewardRow>,
}

///
/// IcNodeProviderRewardInfoSourceData
///
/// Raw exact node-provider reward record and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeProviderRewardInfoSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Raw reward record returned by the source.
    pub reward: IcNodeProviderRewardRow,
}

///
/// IcNodeProviderRewardHistorySourceData
///
/// Raw bounded node-provider reward history and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeProviderRewardHistorySourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// History query applied by the source.
    pub query: IcNodeProviderRewardHistoryQuery,
    /// Raw aggregate observations returned by the source.
    pub observations: Vec<IcNodeProviderRewardHistoryObservation>,
}

///
/// IcReplicaVersionListSourceData
///
/// Raw bounded replica-version page and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcReplicaVersionListSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Page query applied by the source.
    pub query: IcReplicaVersionListQuery,
    /// Proposal-index ceiling selected by the source.
    pub resolved_max_proposal_index: u64,
    /// Number of release records matching the selected ceiling.
    pub total_proposals: u64,
    /// Raw release rows returned by the source.
    pub rows: Vec<IcReplicaVersionListRow>,
}

///
/// IcReplicaVersionInfoSourceData
///
/// Raw exact replica-version record and provenance returned by a Dashboard source.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcReplicaVersionInfoSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Replica-version identifier returned by the source.
    pub replica_version_id: String,
    /// NNS proposal that elected this version.
    pub proposal_id: u64,
    /// Election proposal execution time as raw Unix seconds.
    pub executed_timestamp_seconds: u64,
    /// Raw proposal title.
    pub title: String,
    /// Raw proposal discussion URL.
    pub url: String,
    /// Raw release-note summary.
    pub summary: String,
    /// Raw Dashboard-recorded Subnet assignments.
    pub subnets: Vec<IcReplicaVersionSubnetRollout>,
}
