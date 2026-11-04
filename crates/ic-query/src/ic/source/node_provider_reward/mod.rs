//! Module: ic::source::node_provider_reward
//!
//! Responsibility: Dashboard node-provider reward source contract, bounds, and projection.
//! Does not own: HTTP transport, shared provenance, CLI paging, or text rendering.
//! Boundary: validates one page, one exact record, or one bounded aggregate history response.

use super::inclusive_observation_count;
use super::invalid_request;
use super::invalid_source;
use super::report_provenance;
use super::validate_canonical_principal;
use super::validate_collection_end;
use super::validate_observation_timestamps;
use super::validate_offset_page;
use super::validate_provenance;
use super::validate_source_text;
use crate::hex::is_lowercase_hex_32_bytes;
use crate::ic::IcHostError;
use crate::ic::IcNodeProviderRewardHistoryQuery;
use crate::ic::IcNodeProviderRewardHistoryReport;
use crate::ic::IcNodeProviderRewardHistorySourceData;
use crate::ic::IcNodeProviderRewardInfoReport;
use crate::ic::IcNodeProviderRewardInfoSourceData;
use crate::ic::IcNodeProviderRewardListQuery;
use crate::ic::IcNodeProviderRewardListReport;
use crate::ic::IcNodeProviderRewardListSourceData;
use crate::ic::IcNodeProviderRewardRow;
use crate::ic::IcSourceRequest;
use crate::ic::MAX_IC_NODE_PROVIDER_REWARD_HISTORY_OBSERVATIONS;
use crate::ic::MAX_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS;
use crate::ic::MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT;
use crate::ic::MIN_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS;
use std::collections::HashSet;

///
/// IcNodeProviderRewardSource
///
/// Source contract for bounded official Dashboard node-provider reward reports.
///

pub trait IcNodeProviderRewardSource: Sync {
    /// Fetch one explicitly bounded reward page without automatically paginating.
    fn fetch_node_provider_reward_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcNodeProviderRewardListQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcNodeProviderRewardListSourceData, IcHostError>>;

    /// Fetch one exact reward record by Dashboard id.
    fn fetch_node_provider_reward_info<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        reward_id: u64,
    ) -> crate::report::SourceFuture<'a, Result<IcNodeProviderRewardInfoSourceData, IcHostError>>;

    /// Fetch one bounded aggregate reward history window.
    fn fetch_node_provider_reward_history<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcNodeProviderRewardHistoryQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcNodeProviderRewardHistorySourceData, IcHostError>>;
}

pub(in crate::ic) fn validate_node_provider_reward_list_query(
    query: &IcNodeProviderRewardListQuery,
) -> Result<(), IcHostError> {
    if (1..=MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT).contains(&query.limit) {
        return Ok(());
    }
    invalid_request(
        "query.limit",
        format!("must be between 1 and {MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT}"),
    )
}

pub(in crate::ic) fn validate_node_provider_reward_history_request(
    now_unix_secs: u64,
    query: &IcNodeProviderRewardHistoryQuery,
) -> Result<(), IcHostError> {
    validate_node_provider_reward_history_query(query)?;
    validate_collection_end(now_unix_secs, query.end_unix_secs)
}

pub(in crate::ic) fn validate_node_provider_reward_history_query(
    query: &IcNodeProviderRewardHistoryQuery,
) -> Result<(), IcHostError> {
    super::validate_query_time_order(query.start_unix_secs, query.end_unix_secs)?;
    if !(MIN_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS
        ..=MAX_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS)
        .contains(&query.step_secs)
    {
        return invalid_request(
            "query.step_secs",
            format!(
                "must be between {MIN_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS} and {MAX_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS}"
            ),
        );
    }
    super::validate_observation_budget(
        query.start_unix_secs,
        query.end_unix_secs,
        query.step_secs,
        MAX_IC_NODE_PROVIDER_REWARD_HISTORY_OBSERVATIONS,
        "",
    )
}

pub(in crate::ic) fn node_provider_reward_list_report_from_source(
    request: &IcSourceRequest,
    query: &IcNodeProviderRewardListQuery,
    source: IcNodeProviderRewardListSourceData,
) -> Result<IcNodeProviderRewardListReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    super::validate_requested_query("node-provider reward", query, &source.query)?;
    super::validate_indexed_page_bounds(
        source.query.max_reward_index,
        source.resolved_max_reward_index,
        "resolved_max_reward_index",
        source.rows.len(),
        source.query.limit,
        "node-provider reward",
    )?;
    let next_offset_hint = validate_offset_page(
        source.query.offset,
        source.total_reward_records,
        source.rows.len(),
        "total_reward_records",
        "node-provider reward page offset overflows u64",
    )?;
    validate_reward_rows(&source.rows)?;

    Ok(IcNodeProviderRewardListReport {
        provenance: report_provenance(source.source),
        query: source.query,
        resolved_max_reward_index: source.resolved_max_reward_index,
        total_reward_records: source.total_reward_records,
        next_offset_hint,
        pages_may_overlap: true,
        rows: source.rows,
    })
}

pub(in crate::ic) fn node_provider_reward_info_report_from_source(
    request: &IcSourceRequest,
    requested_reward_id: u64,
    source: IcNodeProviderRewardInfoSourceData,
) -> Result<IcNodeProviderRewardInfoReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_reward_row(&source.reward)?;
    if source.reward.reward_id != requested_reward_id {
        return invalid_source(format!(
            "reward_id is {}, expected requested value {requested_reward_id}",
            source.reward.reward_id
        ));
    }
    Ok(IcNodeProviderRewardInfoReport {
        provenance: report_provenance(source.source),
        reward: source.reward,
    })
}

pub(in crate::ic) fn node_provider_reward_history_report_from_source(
    request: &IcSourceRequest,
    query: &IcNodeProviderRewardHistoryQuery,
    source: IcNodeProviderRewardHistorySourceData,
) -> Result<IcNodeProviderRewardHistoryReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    super::validate_requested_query("node-provider reward history", query, &source.query)?;
    let requested_observation_limit =
        inclusive_observation_count(query.start_unix_secs, query.end_unix_secs, query.step_secs);
    if u64::try_from(source.observations.len()).unwrap_or(u64::MAX) > requested_observation_limit {
        return invalid_source(format!(
            "source returned {} reward-history observations for a request bounded to {requested_observation_limit}",
            source.observations.len()
        ));
    }
    validate_observation_timestamps(
        source
            .observations
            .iter()
            .map(|observation| observation.timestamp_unix_secs),
        query.start_unix_secs..=query.end_unix_secs,
        "reward-history",
    )?;

    Ok(IcNodeProviderRewardHistoryReport {
        provenance: report_provenance(source.source),
        query: source.query,
        requested_observation_limit,
        observations: source.observations,
    })
}

fn validate_reward_rows(rows: &[IcNodeProviderRewardRow]) -> Result<(), IcHostError> {
    let mut seen = HashSet::with_capacity(rows.len());
    for row in rows {
        if !seen.insert(row.reward_id) {
            return invalid_source(format!(
                "duplicate node-provider reward id {}",
                row.reward_id
            ));
        }
        validate_reward_row(row)?;
    }
    Ok(())
}

fn validate_reward_row(row: &IcNodeProviderRewardRow) -> Result<(), IcHostError> {
    validate_canonical_principal("reward.node_provider_id", &row.node_provider_id)?;
    if row.reward_timestamp_unix_secs == 0 {
        return invalid_source("node-provider reward timestamp must be positive");
    }
    validate_source_text("reward.reward_mode", &row.reward_mode, 128)?;
    validate_source_text(
        "reward.dashboard_updated_at",
        &row.dashboard_updated_at,
        128,
    )?;
    if row.proposal_id == Some(0) {
        return invalid_source("node-provider reward proposal_id must be positive when present");
    }
    if let Some(account) = row.details.get("to_account") {
        let Some(account) = account.as_str() else {
            return invalid_source("reward.details.to_account must be a string when present");
        };
        if !is_lowercase_hex_32_bytes(account) {
            return invalid_source(
                "reward.details.to_account must be 64 lowercase hexadecimal characters",
            );
        }
    }
    match (
        row.xdr_conversion_rate.timestamp_unix_secs,
        row.xdr_conversion_rate.xdr_permyriad_per_icp,
    ) {
        (None, None) => {}
        (Some(timestamp), Some(rate)) if timestamp > 0 && rate > 0 => {}
        (Some(_), Some(_)) => {
            return invalid_source("reward XDR conversion-rate values must be positive");
        }
        _ => {
            return invalid_source(
                "reward XDR conversion-rate timestamp and value must both be present or absent",
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
