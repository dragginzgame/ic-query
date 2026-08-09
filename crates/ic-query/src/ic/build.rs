//! Module: ic::build
//!
//! Responsibility: build IC Dashboard reports through focused source capabilities.
//! Does not own: HTTP transport, source result validation, command parsing, or rendering.
//! Boundary: validates request identity before any live source call.

use crate::ic::{
    IcBoundaryNodeDataCentersReport, IcBoundaryNodeDataCentersRequest, IcCanisterCollectionSource,
    IcCanisterCountReport, IcCanisterCountRequest, IcCanisterPageReport, IcCanisterPageRequest,
    IcCanisterReport, IcCanisterRequest, IcCanisterSource, IcDailyStatsReport, IcDailyStatsRequest,
    IcHostError, IcIcrcAccountInfoReport, IcIcrcAccountInfoRequest, IcIcrcAccountListReport,
    IcIcrcAccountListRequest, IcIcrcAnalyticsRequest, IcIcrcAnalyticsSource,
    IcIcrcHolderListReport, IcIcrcHolderListRequest, IcIcrcIndexSource, IcIcrcIndexedCountReport,
    IcIcrcIndexedCountRequest, IcIcrcTokenValueReport, IcIcrcTokenValueRequest,
    IcIcrcTotalSupplyReport, IcIcrcTotalSupplyRequest, IcMetricReport, IcMetricRequest,
    IcMetricSource, IcNetworkSource, IcNodeProviderRewardHistoryReport,
    IcNodeProviderRewardHistoryRequest, IcNodeProviderRewardInfoReport,
    IcNodeProviderRewardInfoRequest, IcNodeProviderRewardListReport,
    IcNodeProviderRewardListRequest, IcNodeProviderRewardSource, IcNodeStatusSnapshot,
    IcNodeStatusSnapshotRequest, IcNodeStatusSource, IcReplicaVersionInfoReport,
    IcReplicaVersionInfoRequest, IcReplicaVersionListReport, IcReplicaVersionListRequest,
    IcReplicaVersionSource, IcSourceRequest, LiveIcSource,
    source::{
        boundary_node_data_centers_report_from_source, canonical_canister_id,
        canonical_page_cursors, canonical_request_principal, count_report_from_source,
        daily_stats_report_from_source, dashboard_source_request,
        icrc_account_info_report_from_source, icrc_account_list_report_from_source,
        icrc_holder_list_report_from_source, icrc_indexed_count_report_from_source,
        icrc_token_value_report_from_source, icrc_total_supply_report_from_source,
        metric_report_from_source, node_provider_reward_history_report_from_source,
        node_provider_reward_info_report_from_source, node_provider_reward_list_report_from_source,
        node_status_snapshot_from_source, normalized_account_list_query, normalized_filters,
        page_report_from_source, replica_version_info_report_from_source,
        replica_version_list_report_from_source, report_from_source, validate_account_id,
        validate_daily_stats_request, validate_holder_list_query,
        validate_icrc_token_value_request, validate_icrc_total_supply_request,
        validate_metric_request, validate_node_provider_reward_history_request,
        validate_node_provider_reward_list_query, validate_page_cursor_exclusivity,
        validate_page_limit, validate_replica_version_id, validate_replica_version_list_query,
    },
};

/// Build one finite live observed node-status snapshot from the official Dashboard API.
pub fn build_ic_node_status_snapshot(
    request: &IcNodeStatusSnapshotRequest,
) -> Result<IcNodeStatusSnapshot, IcHostError> {
    build_ic_node_status_snapshot_with_source(request, &LiveIcSource)
}

/// Build one finite observed node-status snapshot through a custom Dashboard source.
pub fn build_ic_node_status_snapshot_with_source(
    request: &IcNodeStatusSnapshotRequest,
    source: &dyn IcNodeStatusSource,
) -> Result<IcNodeStatusSnapshot, IcHostError> {
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_node_status_snapshot(&source_request)?;
    node_status_snapshot_from_source(&source_request, source_data)
}

/// Build one live boundary-node data-center report from the official Dashboard API.
pub fn build_ic_boundary_node_data_centers_report(
    request: &IcBoundaryNodeDataCentersRequest,
) -> Result<IcBoundaryNodeDataCentersReport, IcHostError> {
    build_ic_boundary_node_data_centers_report_with_source(request, &LiveIcSource)
}

/// Build one boundary-node data-center report through a custom Dashboard source.
pub fn build_ic_boundary_node_data_centers_report_with_source(
    request: &IcBoundaryNodeDataCentersRequest,
    source: &dyn IcNetworkSource,
) -> Result<IcBoundaryNodeDataCentersReport, IcHostError> {
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_boundary_node_data_centers(&source_request)?;
    boundary_node_data_centers_report_from_source(&source_request, source_data)
}

/// Build one live, bounded daily network-activity report from the official Dashboard API.
pub fn build_ic_daily_stats_report(
    request: &IcDailyStatsRequest,
) -> Result<IcDailyStatsReport, IcHostError> {
    build_ic_daily_stats_report_with_source(request, &LiveIcSource)
}

/// Build one bounded daily network-activity report through a custom Dashboard source.
pub fn build_ic_daily_stats_report_with_source(
    request: &IcDailyStatsRequest,
    source: &dyn IcNetworkSource,
) -> Result<IcDailyStatsReport, IcHostError> {
    validate_daily_stats_request(request.now_unix_secs, &request.query)?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_daily_stats(&source_request, &request.query)?;
    daily_stats_report_from_source(&source_request, &request.query, source_data)
}

/// Build one live, bounded node-provider reward page from the official Dashboard API.
pub fn build_ic_node_provider_reward_list_report(
    request: &IcNodeProviderRewardListRequest,
) -> Result<IcNodeProviderRewardListReport, IcHostError> {
    build_ic_node_provider_reward_list_report_with_source(request, &LiveIcSource)
}

/// Build one bounded node-provider reward page through a custom Dashboard source.
pub fn build_ic_node_provider_reward_list_report_with_source(
    request: &IcNodeProviderRewardListRequest,
    source: &dyn IcNodeProviderRewardSource,
) -> Result<IcNodeProviderRewardListReport, IcHostError> {
    validate_node_provider_reward_list_query(&request.query)?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_node_provider_reward_list(&source_request, &request.query)?;
    node_provider_reward_list_report_from_source(&source_request, &request.query, source_data)
}

/// Build one live exact node-provider reward report from the official Dashboard API.
pub fn build_ic_node_provider_reward_info_report(
    request: &IcNodeProviderRewardInfoRequest,
) -> Result<IcNodeProviderRewardInfoReport, IcHostError> {
    build_ic_node_provider_reward_info_report_with_source(request, &LiveIcSource)
}

/// Build one exact node-provider reward report through a custom Dashboard source.
pub fn build_ic_node_provider_reward_info_report_with_source(
    request: &IcNodeProviderRewardInfoRequest,
    source: &dyn IcNodeProviderRewardSource,
) -> Result<IcNodeProviderRewardInfoReport, IcHostError> {
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_node_provider_reward_info(&source_request, request.reward_id)?;
    node_provider_reward_info_report_from_source(&source_request, request.reward_id, source_data)
}

/// Build one live, bounded node-provider reward history from the official Dashboard API.
pub fn build_ic_node_provider_reward_history_report(
    request: &IcNodeProviderRewardHistoryRequest,
) -> Result<IcNodeProviderRewardHistoryReport, IcHostError> {
    build_ic_node_provider_reward_history_report_with_source(request, &LiveIcSource)
}

/// Build one bounded node-provider reward history through a custom Dashboard source.
pub fn build_ic_node_provider_reward_history_report_with_source(
    request: &IcNodeProviderRewardHistoryRequest,
    source: &dyn IcNodeProviderRewardSource,
) -> Result<IcNodeProviderRewardHistoryReport, IcHostError> {
    validate_node_provider_reward_history_request(request.now_unix_secs, &request.query)?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_node_provider_reward_history(&source_request, &request.query)?;
    node_provider_reward_history_report_from_source(&source_request, &request.query, source_data)
}

/// Build one live, bounded replica-version page from the official Dashboard API.
pub fn build_ic_replica_version_list_report(
    request: &IcReplicaVersionListRequest,
) -> Result<IcReplicaVersionListReport, IcHostError> {
    build_ic_replica_version_list_report_with_source(request, &LiveIcSource)
}

/// Build one bounded replica-version page through a custom Dashboard source.
pub fn build_ic_replica_version_list_report_with_source(
    request: &IcReplicaVersionListRequest,
    source: &dyn IcReplicaVersionSource,
) -> Result<IcReplicaVersionListReport, IcHostError> {
    validate_replica_version_list_query(&request.query)?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_replica_version_list(&source_request, &request.query)?;
    replica_version_list_report_from_source(&source_request, &request.query, source_data)
}

/// Build one live exact replica-version report from the official Dashboard API.
pub fn build_ic_replica_version_info_report(
    request: &IcReplicaVersionInfoRequest,
) -> Result<IcReplicaVersionInfoReport, IcHostError> {
    build_ic_replica_version_info_report_with_source(request, &LiveIcSource)
}

/// Build one exact replica-version report through a custom Dashboard source.
pub fn build_ic_replica_version_info_report_with_source(
    request: &IcReplicaVersionInfoRequest,
    source: &dyn IcReplicaVersionSource,
) -> Result<IcReplicaVersionInfoReport, IcHostError> {
    validate_replica_version_id(&request.replica_version_id)?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data =
        source.fetch_replica_version_info(&source_request, &request.replica_version_id)?;
    replica_version_info_report_from_source(
        &source_request,
        &request.replica_version_id,
        source_data,
    )
}

/// Build one live, bounded metric report from the official Dashboard Metrics API.
pub fn build_ic_metric_report(request: &IcMetricRequest) -> Result<IcMetricReport, IcHostError> {
    build_ic_metric_report_with_source(request, &LiveIcSource)
}

/// Build one bounded metric report through a custom Dashboard source capability.
pub fn build_ic_metric_report_with_source(
    request: &IcMetricRequest,
    source: &dyn IcMetricSource,
) -> Result<IcMetricReport, IcHostError> {
    validate_metric_request(request.now_unix_secs, &request.query)?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_metric(&source_request, &request.query)?;
    metric_report_from_source(&source_request, &request.query, source_data)
}

/// Build one live, bounded total-supply series from the official Dashboard ICRC API.
pub fn build_icrc_total_supply_report(
    request: &IcIcrcTotalSupplyRequest,
) -> Result<IcIcrcTotalSupplyReport, IcHostError> {
    build_icrc_total_supply_report_with_source(request, &LiveIcSource)
}

/// Build one bounded ICRC total-supply series through a custom Dashboard source capability.
pub fn build_icrc_total_supply_report_with_source(
    request: &IcIcrcTotalSupplyRequest,
    source: &dyn IcIcrcAnalyticsSource,
) -> Result<IcIcrcTotalSupplyReport, IcHostError> {
    validate_icrc_total_supply_request(request.analytics.now_unix_secs, &request.query)?;
    let (source_request, ledger_canister_id) = icrc_analytics_target(&request.analytics)?;
    let source_data =
        source.fetch_total_supply_series(&source_request, &ledger_canister_id, &request.query)?;
    icrc_total_supply_report_from_source(
        &source_request,
        &ledger_canister_id,
        &request.query,
        source_data,
    )
}

/// Build one live indexed-count report from the official Dashboard ICRC API.
pub fn build_icrc_indexed_count_report(
    request: &IcIcrcIndexedCountRequest,
) -> Result<IcIcrcIndexedCountReport, IcHostError> {
    build_icrc_indexed_count_report_with_source(request, &LiveIcSource)
}

/// Build one indexed-count report through a custom Dashboard source capability.
pub fn build_icrc_indexed_count_report_with_source(
    request: &IcIcrcIndexedCountRequest,
    source: &dyn IcIcrcAnalyticsSource,
) -> Result<IcIcrcIndexedCountReport, IcHostError> {
    let (source_request, ledger_canister_id) = icrc_analytics_target(&request.analytics)?;
    let source_data =
        source.fetch_indexed_count(&source_request, &ledger_canister_id, request.kind)?;
    icrc_indexed_count_report_from_source(
        &source_request,
        &ledger_canister_id,
        request.kind,
        source_data,
    )
}

/// Build one live, bounded account-index page from the official Dashboard ICRC API.
pub fn build_icrc_account_list_report(
    request: &IcIcrcAccountListRequest,
) -> Result<IcIcrcAccountListReport, IcHostError> {
    build_icrc_account_list_report_with_source(request, &LiveIcSource)
}

/// Build one bounded account-index page through a custom Dashboard source capability.
pub fn build_icrc_account_list_report_with_source(
    request: &IcIcrcAccountListRequest,
    source: &dyn IcIcrcIndexSource,
) -> Result<IcIcrcAccountListReport, IcHostError> {
    let query = normalized_account_list_query(&request.query)?;
    let (source_request, ledger_canister_id) = icrc_analytics_target(&request.analytics)?;
    let source_data = source.fetch_account_list(&source_request, &ledger_canister_id, &query)?;
    icrc_account_list_report_from_source(&source_request, &ledger_canister_id, &query, source_data)
}

/// Build one live exact account record from the official Dashboard ICRC API.
pub fn build_icrc_account_info_report(
    request: &IcIcrcAccountInfoRequest,
) -> Result<IcIcrcAccountInfoReport, IcHostError> {
    build_icrc_account_info_report_with_source(request, &LiveIcSource)
}

/// Build one exact account record through a custom Dashboard source capability.
pub fn build_icrc_account_info_report_with_source(
    request: &IcIcrcAccountInfoRequest,
    source: &dyn IcIcrcIndexSource,
) -> Result<IcIcrcAccountInfoReport, IcHostError> {
    validate_account_id(&request.account_id)?;
    let (source_request, ledger_canister_id) = icrc_analytics_target(&request.analytics)?;
    let source_data =
        source.fetch_account_info(&source_request, &ledger_canister_id, &request.account_id)?;
    icrc_account_info_report_from_source(
        &source_request,
        &ledger_canister_id,
        &request.account_id,
        source_data,
    )
}

/// Build one live, bounded holder-index page from the official Dashboard ICRC API.
pub fn build_icrc_holder_list_report(
    request: &IcIcrcHolderListRequest,
) -> Result<IcIcrcHolderListReport, IcHostError> {
    build_icrc_holder_list_report_with_source(request, &LiveIcSource)
}

/// Build one bounded holder-index page through a custom Dashboard source capability.
pub fn build_icrc_holder_list_report_with_source(
    request: &IcIcrcHolderListRequest,
    source: &dyn IcIcrcIndexSource,
) -> Result<IcIcrcHolderListReport, IcHostError> {
    validate_holder_list_query(&request.query)?;
    let (source_request, ledger_canister_id) = icrc_analytics_target(&request.analytics)?;
    let source_data =
        source.fetch_holder_list(&source_request, &ledger_canister_id, &request.query)?;
    icrc_holder_list_report_from_source(
        &source_request,
        &ledger_canister_id,
        &request.query,
        source_data,
    )
}

/// Build one live, bounded token-value series from the official Dashboard ICRC API.
pub fn build_icrc_token_value_report(
    request: &IcIcrcTokenValueRequest,
) -> Result<IcIcrcTokenValueReport, IcHostError> {
    build_icrc_token_value_report_with_source(request, &LiveIcSource)
}

/// Build one bounded token-value series through a custom Dashboard source capability.
pub fn build_icrc_token_value_report_with_source(
    request: &IcIcrcTokenValueRequest,
    source: &dyn IcIcrcAnalyticsSource,
) -> Result<IcIcrcTokenValueReport, IcHostError> {
    validate_icrc_token_value_request(request.analytics.now_unix_secs, &request.query)?;
    let (source_request, ledger_canister_id) = icrc_analytics_target(&request.analytics)?;
    let source_data =
        source.fetch_token_value_series(&source_request, &ledger_canister_id, &request.query)?;
    icrc_token_value_report_from_source(
        &source_request,
        &ledger_canister_id,
        &request.query,
        source_data,
    )
}

fn icrc_analytics_target(
    request: &IcIcrcAnalyticsRequest,
) -> Result<(IcSourceRequest, String), IcHostError> {
    let ledger_canister_id =
        canonical_request_principal("ledger_canister_id", &request.ledger_canister_id)?;
    Ok((
        dashboard_source_request(&request.source_endpoint, request.now_unix_secs),
        ledger_canister_id,
    ))
}

/// Build one live canister report from the official IC Dashboard API.
pub fn build_ic_canister_report(
    request: &IcCanisterRequest,
) -> Result<IcCanisterReport, IcHostError> {
    build_ic_canister_report_with_source(request, &LiveIcSource)
}

/// Build one canister report through a custom Dashboard source capability.
pub fn build_ic_canister_report_with_source(
    request: &IcCanisterRequest,
    source: &dyn IcCanisterSource,
) -> Result<IcCanisterReport, IcHostError> {
    let canister_id = canonical_canister_id(&request.canister_id)?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_canister(&source_request, &canister_id)?;
    report_from_source(&source_request, &canister_id, source_data)
}

/// Build one live filtered canister count from the official IC Dashboard API.
pub fn build_ic_canister_count_report(
    request: &IcCanisterCountRequest,
) -> Result<IcCanisterCountReport, IcHostError> {
    build_ic_canister_count_report_with_source(request, &LiveIcSource)
}

/// Build one filtered canister count through a custom Dashboard source capability.
pub fn build_ic_canister_count_report_with_source(
    request: &IcCanisterCountRequest,
    source: &dyn IcCanisterCollectionSource,
) -> Result<IcCanisterCountReport, IcHostError> {
    let filters = normalized_filters(&request.filters)?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_canister_count(&source_request, &filters)?;
    count_report_from_source(&source_request, &filters, source_data)
}

/// Build one live, bounded canister page from the official IC Dashboard API.
pub fn build_ic_canister_page_report(
    request: &IcCanisterPageRequest,
) -> Result<IcCanisterPageReport, IcHostError> {
    build_ic_canister_page_report_with_source(request, &LiveIcSource)
}

/// Build one bounded canister page through a custom Dashboard source capability.
pub fn build_ic_canister_page_report_with_source(
    request: &IcCanisterPageRequest,
    source: &dyn IcCanisterCollectionSource,
) -> Result<IcCanisterPageReport, IcHostError> {
    validate_page_limit(request.limit)?;
    validate_page_cursor_exclusivity(request.after.as_deref(), request.before.as_deref())?;
    let filters = normalized_filters(&request.filters)?;
    let (after, before) =
        canonical_page_cursors(request.after.as_deref(), request.before.as_deref())?;
    let source_request = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_canister_page(
        &source_request,
        &filters,
        request.limit,
        after.as_deref(),
        before.as_deref(),
    )?;
    page_report_from_source(
        &source_request,
        &filters,
        request.limit,
        after.as_deref(),
        before.as_deref(),
        source_data,
    )
}
