//! Module: ic::source::node_provider_reward
//!
//! Responsibility: Dashboard node-provider reward source contract, bounds, and projection.
//! Does not own: HTTP transport, shared provenance, CLI paging, or text rendering.
//! Boundary: validates one page, one exact record, or one bounded aggregate history response.

use super::{
    inclusive_observation_count, invalid_request, invalid_source, report_provenance,
    validate_canonical_principal, validate_collection_end, validate_provenance,
};
use crate::{
    hex::is_lowercase_hex,
    ic::{
        IcHostError, IcNodeProviderRewardHistoryQuery, IcNodeProviderRewardHistoryReport,
        IcNodeProviderRewardHistorySourceData, IcNodeProviderRewardInfoReport,
        IcNodeProviderRewardInfoSourceData, IcNodeProviderRewardListQuery,
        IcNodeProviderRewardListReport, IcNodeProviderRewardListSourceData,
        IcNodeProviderRewardRow, IcSourceRequest, MAX_IC_NODE_PROVIDER_REWARD_HISTORY_OBSERVATIONS,
        MAX_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS, MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT,
        MIN_IC_NODE_PROVIDER_REWARD_HISTORY_STEP_SECS,
    },
};
use std::collections::HashSet;

///
/// IcNodeProviderRewardSource
///
/// Source contract for bounded official Dashboard node-provider reward reports.
///

pub trait IcNodeProviderRewardSource {
    /// Fetch one explicitly bounded reward page without automatically paginating.
    fn fetch_node_provider_reward_list(
        &self,
        request: &IcSourceRequest,
        query: &IcNodeProviderRewardListQuery,
    ) -> Result<IcNodeProviderRewardListSourceData, IcHostError>;

    /// Fetch one exact reward record by Dashboard id.
    fn fetch_node_provider_reward_info(
        &self,
        request: &IcSourceRequest,
        reward_id: u64,
    ) -> Result<IcNodeProviderRewardInfoSourceData, IcHostError>;

    /// Fetch one bounded aggregate reward history window.
    fn fetch_node_provider_reward_history(
        &self,
        request: &IcSourceRequest,
        query: &IcNodeProviderRewardHistoryQuery,
    ) -> Result<IcNodeProviderRewardHistorySourceData, IcHostError>;
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
    if query.end_unix_secs < query.start_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must be greater than or equal to query.start_unix_secs",
        );
    }
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
    let observations =
        inclusive_observation_count(query.start_unix_secs, query.end_unix_secs, query.step_secs);
    if observations > MAX_IC_NODE_PROVIDER_REWARD_HISTORY_OBSERVATIONS {
        return invalid_request(
            "query",
            format!(
                "would request {observations} observations; maximum is {MAX_IC_NODE_PROVIDER_REWARD_HISTORY_OBSERVATIONS}"
            ),
        );
    }
    Ok(())
}

pub(in crate::ic) fn node_provider_reward_list_report_from_source(
    request: &IcSourceRequest,
    query: &IcNodeProviderRewardListQuery,
    source: IcNodeProviderRewardListSourceData,
) -> Result<IcNodeProviderRewardListReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    if source.query != *query {
        return invalid_source(format!(
            "node-provider reward query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }
    validate_node_provider_reward_list_query(&source.query)?;
    if let Some(requested_maximum) = source.query.max_reward_index
        && source.resolved_max_reward_index > requested_maximum
    {
        return invalid_source(format!(
            "resolved_max_reward_index is {}, greater than requested ceiling {requested_maximum}",
            source.resolved_max_reward_index
        ));
    }
    if source.rows.len() > usize::from(source.query.limit) {
        return invalid_source(format!(
            "source returned {} node-provider reward rows for requested limit {}",
            source.rows.len(),
            source.query.limit
        ));
    }
    validate_list_metadata(
        &source.query,
        source.total_reward_records,
        source.rows.len(),
    )?;
    validate_reward_rows(&source.rows)?;

    let returned_count = source.rows.len();
    let consumed = source
        .query
        .offset
        .checked_add(u64::try_from(returned_count).unwrap_or(u64::MAX))
        .ok_or_else(|| IcHostError::InvalidSourceData {
            reason: "node-provider reward page offset overflows u64".to_string(),
        })?;
    let next_offset_hint =
        (returned_count > 0 && consumed < source.total_reward_records).then_some(consumed);

    Ok(IcNodeProviderRewardListReport {
        provenance: report_provenance(source.source),
        query: source.query,
        resolved_max_reward_index: source.resolved_max_reward_index,
        total_reward_records: source.total_reward_records,
        returned_count,
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
    if source.query != *query {
        return invalid_source(format!(
            "node-provider reward history query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }
    let requested_observation_limit =
        inclusive_observation_count(query.start_unix_secs, query.end_unix_secs, query.step_secs);
    if u64::try_from(source.observations.len()).unwrap_or(u64::MAX) > requested_observation_limit {
        return invalid_source(format!(
            "source returned {} reward-history observations for a request bounded to {requested_observation_limit}",
            source.observations.len()
        ));
    }
    let mut previous_timestamp = None;
    for observation in &source.observations {
        if !(query.start_unix_secs..=query.end_unix_secs).contains(&observation.timestamp_unix_secs)
        {
            return invalid_source(format!(
                "reward-history observation timestamp {} is outside the requested window",
                observation.timestamp_unix_secs
            ));
        }
        if previous_timestamp.is_some_and(|previous| previous >= observation.timestamp_unix_secs) {
            return invalid_source(
                "reward-history observations must be strictly ordered by timestamp",
            );
        }
        previous_timestamp = Some(observation.timestamp_unix_secs);
    }

    Ok(IcNodeProviderRewardHistoryReport {
        provenance: report_provenance(source.source),
        query: source.query,
        requested_observation_limit,
        returned_observation_count: source.observations.len(),
        observations: source.observations,
    })
}

fn validate_list_metadata(
    query: &IcNodeProviderRewardListQuery,
    total_reward_records: u64,
    returned_count: usize,
) -> Result<(), IcHostError> {
    let returned_count = u64::try_from(returned_count).unwrap_or(u64::MAX);
    if returned_count > total_reward_records {
        return invalid_source(format!(
            "returned_count {returned_count} exceeds total_reward_records {total_reward_records}"
        ));
    }
    if returned_count == 0 {
        return Ok(());
    }
    if query.offset >= total_reward_records {
        return invalid_source(format!(
            "nonempty page starts at offset {}, but total_reward_records is {total_reward_records}",
            query.offset
        ));
    }
    let consumed =
        query
            .offset
            .checked_add(returned_count)
            .ok_or_else(|| IcHostError::InvalidSourceData {
                reason: "node-provider reward page offset overflows u64".to_string(),
            })?;
    if consumed > total_reward_records {
        return invalid_source(format!(
            "page ending at offset {consumed} exceeds total_reward_records {total_reward_records}"
        ));
    }
    Ok(())
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
    validate_text("reward.reward_mode", &row.reward_mode, 128)?;
    validate_text(
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
        if account.len() != 64 || !is_lowercase_hex(account) {
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

fn validate_text(field: &'static str, value: &str, max_bytes: usize) -> Result<(), IcHostError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.len() > max_bytes
        || value.chars().any(char::is_control)
    {
        return invalid_source(format!(
            "{field} must be nonempty trimmed text of at most {max_bytes} bytes without control characters"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ic::{
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcNodeProviderRewardHistoryObservation,
        IcNodeProviderRewardHistoryRequest, IcNodeProviderRewardInfoRequest,
        IcNodeProviderRewardListRequest, IcNodeProviderRewardXdrConversionRate,
        build_ic_node_provider_reward_history_report_with_source,
        build_ic_node_provider_reward_info_report_with_source,
        build_ic_node_provider_reward_list_report_with_source,
        ic_node_provider_reward_history_report_text, ic_node_provider_reward_list_report_text,
    };
    use std::{cell::Cell, collections::BTreeMap};

    #[test]
    fn builders_make_one_call_and_preserve_explicit_contracts() {
        let source = FixtureSource::default();
        let list = build_ic_node_provider_reward_list_report_with_source(
            &IcNodeProviderRewardListRequest::new(
                DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
                1_800_000_000,
                IcNodeProviderRewardListQuery::new(2, 0, Some(6_470)),
            ),
            &source,
        )
        .expect("reward list");
        let info = build_ic_node_provider_reward_info_report_with_source(
            &IcNodeProviderRewardInfoRequest::new(
                DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
                1_800_000_000,
                7_562,
            ),
            &source,
        )
        .expect("reward info");
        let history = build_ic_node_provider_reward_history_report_with_source(
            &IcNodeProviderRewardHistoryRequest::new(
                DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
                1_800_000_000,
                IcNodeProviderRewardHistoryQuery::new(1_783_900_000, 1_784_300_000, 86_400),
            ),
            &source,
        )
        .expect("reward history");

        assert_eq!(source.calls.get(), 3);
        assert_eq!(list.returned_count, 1);
        assert_eq!(list.next_offset_hint, Some(1));
        assert!(list.pages_may_overlap);
        assert_eq!(info.reward.reward_id, 7_562);
        assert_eq!(history.requested_observation_limit, 5);
        assert_eq!(history.returned_observation_count, 1);
        let list_text = ic_node_provider_reward_list_report_text(&list);
        let (_, table) = list_text
            .split_once("\n\n")
            .expect("reward preamble and table are separate sections");
        assert!(table.contains("ID"));
        assert!(ic_node_provider_reward_history_report_text(&history).contains("AMOUNT_E8S"));
    }

    #[test]
    fn invalid_requests_fail_before_custom_source_calls() {
        let source = FixtureSource::default();
        let error = build_ic_node_provider_reward_list_report_with_source(
            &IcNodeProviderRewardListRequest::new(
                DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
                1_800_000_000,
                IcNodeProviderRewardListQuery::new(0, 0, None),
            ),
            &source,
        )
        .expect_err("zero limit must fail");
        assert!(matches!(error, IcHostError::InvalidRequest { .. }));

        let error = build_ic_node_provider_reward_history_report_with_source(
            &IcNodeProviderRewardHistoryRequest::new(
                DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
                1_700_000_000,
                IcNodeProviderRewardHistoryQuery::new(1_700_000_000, 1_700_000_001, 60),
            ),
            &source,
        )
        .expect_err("future end must fail");
        assert!(matches!(error, IcHostError::InvalidRequest { .. }));
        assert_eq!(source.calls.get(), 0);
    }

    #[test]
    fn duplicate_rows_unordered_history_and_wrong_exact_id_are_rejected() {
        let request = source_request();
        let query = IcNodeProviderRewardListQuery::new(2, 0, None);
        let error = node_provider_reward_list_report_from_source(
            &request,
            &query,
            IcNodeProviderRewardListSourceData {
                source: request.clone(),
                query: query.clone(),
                resolved_max_reward_index: 2,
                total_reward_records: 2,
                rows: vec![reward_row(7_562), reward_row(7_562)],
            },
        )
        .expect_err("duplicate ids must fail");
        assert!(matches!(error, IcHostError::InvalidSourceData { .. }));

        let history_query =
            IcNodeProviderRewardHistoryQuery::new(1_783_900_000, 1_784_300_000, 86_400);
        let error = node_provider_reward_history_report_from_source(
            &request,
            &history_query,
            IcNodeProviderRewardHistorySourceData {
                source: request.clone(),
                query: history_query.clone(),
                observations: vec![
                    IcNodeProviderRewardHistoryObservation {
                        timestamp_unix_secs: 1_784_073_600,
                        amount_e8s: 2,
                    },
                    IcNodeProviderRewardHistoryObservation {
                        timestamp_unix_secs: 1_784_000_000,
                        amount_e8s: 1,
                    },
                ],
            },
        )
        .expect_err("unordered history must fail");
        assert!(matches!(error, IcHostError::InvalidSourceData { .. }));

        let error = node_provider_reward_info_report_from_source(
            &request,
            1,
            IcNodeProviderRewardInfoSourceData {
                source: request.clone(),
                reward: reward_row(2),
            },
        )
        .expect_err("wrong exact id must fail");
        assert!(matches!(error, IcHostError::InvalidSourceData { .. }));
    }

    #[derive(Default)]
    struct FixtureSource {
        calls: Cell<usize>,
    }

    impl IcNodeProviderRewardSource for FixtureSource {
        fn fetch_node_provider_reward_list(
            &self,
            request: &IcSourceRequest,
            query: &IcNodeProviderRewardListQuery,
        ) -> Result<IcNodeProviderRewardListSourceData, IcHostError> {
            self.calls.set(self.calls.get() + 1);
            Ok(IcNodeProviderRewardListSourceData {
                source: request.clone(),
                query: query.clone(),
                resolved_max_reward_index: 6_470,
                total_reward_records: 6_470,
                rows: vec![reward_row(7_562)],
            })
        }

        fn fetch_node_provider_reward_info(
            &self,
            request: &IcSourceRequest,
            reward_id: u64,
        ) -> Result<IcNodeProviderRewardInfoSourceData, IcHostError> {
            self.calls.set(self.calls.get() + 1);
            Ok(IcNodeProviderRewardInfoSourceData {
                source: request.clone(),
                reward: reward_row(reward_id),
            })
        }

        fn fetch_node_provider_reward_history(
            &self,
            request: &IcSourceRequest,
            query: &IcNodeProviderRewardHistoryQuery,
        ) -> Result<IcNodeProviderRewardHistorySourceData, IcHostError> {
            self.calls.set(self.calls.get() + 1);
            Ok(IcNodeProviderRewardHistorySourceData {
                source: request.clone(),
                query: query.clone(),
                observations: vec![IcNodeProviderRewardHistoryObservation {
                    timestamp_unix_secs: 1_784_073_600,
                    amount_e8s: 66_852_931_445_000,
                }],
            })
        }
    }

    fn source_request() -> IcSourceRequest {
        IcSourceRequest::new(
            DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
            "2026-08-08T00:00:00Z",
            "test",
        )
    }

    fn reward_row(reward_id: u64) -> IcNodeProviderRewardRow {
        IcNodeProviderRewardRow {
            reward_id,
            amount_e8s: 1_583_574_085_000,
            details: BTreeMap::from([(
                "to_account".to_string(),
                serde_json::Value::String("00".repeat(32)),
            )]),
            maximum_node_provider_rewards_e8s: Some(10_000_000_000_000),
            minimum_xdr_permyriad_per_icp: Some(20_000),
            node_provider_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
            proposal_id: None,
            registry_version: None,
            reward_mode: "RewardToAccount".to_string(),
            reward_timestamp_unix_secs: 1_784_081_341,
            dashboard_updated_at: "2026-07-15T04:30:01.558435".to_string(),
            xdr_conversion_rate: IcNodeProviderRewardXdrConversionRate {
                timestamp_unix_secs: Some(1_784_073_600),
                xdr_permyriad_per_icp: Some(16_379),
            },
        }
    }
}
