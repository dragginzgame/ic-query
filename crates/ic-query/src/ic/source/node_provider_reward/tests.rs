use super::*;
use crate::ic::DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT;
use crate::ic::IcNodeProviderRewardHistoryObservation;
use crate::ic::IcNodeProviderRewardHistoryRequest;
use crate::ic::IcNodeProviderRewardInfoRequest;
use crate::ic::IcNodeProviderRewardListRequest;
use crate::ic::IcNodeProviderRewardXdrConversionRate;
use crate::ic::build_ic_node_provider_reward_history_report_with_source;
use crate::ic::build_ic_node_provider_reward_info_report_with_source;
use crate::ic::build_ic_node_provider_reward_list_report_with_source;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[tokio::test(flavor = "current_thread")]
async fn builders_make_one_call_and_preserve_explicit_contracts() {
    let source = FixtureSource::default();
    let list = build_ic_node_provider_reward_list_report_with_source(
        &IcNodeProviderRewardListRequest::new(
            crate::ic::IcDashboardRequest::new(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, 1_800_000_000),
            IcNodeProviderRewardListQuery::new(2, 0, Some(6_470)),
        ),
        &source,
    )
    .await
    .expect("reward list");
    let info = build_ic_node_provider_reward_info_report_with_source(
        &IcNodeProviderRewardInfoRequest::new(
            crate::ic::IcDashboardRequest::new(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, 1_800_000_000),
            7_562,
        ),
        &source,
    )
    .await
    .expect("reward info");
    let history = build_ic_node_provider_reward_history_report_with_source(
        &IcNodeProviderRewardHistoryRequest::new(
            crate::ic::IcDashboardRequest::new(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, 1_800_000_000),
            IcNodeProviderRewardHistoryQuery::new(1_783_900_000, 1_784_300_000, 86_400),
        ),
        &source,
    )
    .await
    .expect("reward history");

    assert_eq!((*source.calls.lock().unwrap()), 3);
    assert_eq!(list.rows.len(), 1);
    assert_eq!(list.next_offset_hint, Some(1));
    assert!(list.pages_may_overlap);
    assert_eq!(info.reward.reward_id, 7_562);
    assert_eq!(history.requested_observation_limit, 5);
    assert_eq!(history.observations.len(), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_requests_fail_before_custom_source_calls() {
    let source = FixtureSource::default();
    for limit in [0, MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT + 1] {
        let error = build_ic_node_provider_reward_list_report_with_source(
            &IcNodeProviderRewardListRequest::new(
                crate::ic::IcDashboardRequest::new(
                    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
                    1_800_000_000,
                ),
                IcNodeProviderRewardListQuery::new(limit, 0, None),
            ),
            &source,
        )
        .await
        .expect_err("out-of-range limit must fail");
        assert!(matches!(error, IcHostError::InvalidRequest { .. }));
    }

    let error = build_ic_node_provider_reward_history_report_with_source(
        &IcNodeProviderRewardHistoryRequest::new(
            crate::ic::IcDashboardRequest::new(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, 1_700_000_000),
            IcNodeProviderRewardHistoryQuery::new(1_700_000_000, 1_700_000_001, 60),
        ),
        &source,
    )
    .await
    .expect_err("future end must fail");
    assert!(matches!(error, IcHostError::InvalidRequest { .. }));
    assert_eq!((*source.calls.lock().unwrap()), 0);
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

    for limit in [0, MAX_IC_NODE_PROVIDER_REWARD_PAGE_LIMIT + 1] {
        let error = node_provider_reward_list_report_from_source(
            &request,
            &query,
            IcNodeProviderRewardListSourceData {
                source: request.clone(),
                query: IcNodeProviderRewardListQuery::new(limit, 0, None),
                resolved_max_reward_index: 2,
                total_reward_records: 2,
                rows: vec![reward_row(7_562)],
            },
        )
        .expect_err("source must echo the requested query");
        assert!(matches!(error, IcHostError::InvalidSourceData { .. }));
    }

    let history_query = IcNodeProviderRewardHistoryQuery::new(1_783_900_000, 1_784_300_000, 86_400);
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
    calls: Mutex<usize>,
}

impl IcNodeProviderRewardSource for FixtureSource {
    fn fetch_node_provider_reward_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcNodeProviderRewardListQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcNodeProviderRewardListSourceData, IcHostError>>
    {
        Box::pin(async move {
            *self.calls.lock().unwrap() += 1;
            Ok(IcNodeProviderRewardListSourceData {
                source: request.clone(),
                query: query.clone(),
                resolved_max_reward_index: 6_470,
                total_reward_records: 6_470,
                rows: vec![reward_row(7_562)],
            })
        })
    }

    fn fetch_node_provider_reward_info<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        reward_id: u64,
    ) -> crate::report::SourceFuture<'a, Result<IcNodeProviderRewardInfoSourceData, IcHostError>>
    {
        Box::pin(async move {
            *self.calls.lock().unwrap() += 1;
            Ok(IcNodeProviderRewardInfoSourceData {
                source: request.clone(),
                reward: reward_row(reward_id),
            })
        })
    }

    fn fetch_node_provider_reward_history<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcNodeProviderRewardHistoryQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcNodeProviderRewardHistorySourceData, IcHostError>>
    {
        Box::pin(async move {
            *self.calls.lock().unwrap() += 1;
            Ok(IcNodeProviderRewardHistorySourceData {
                source: request.clone(),
                query: query.clone(),
                observations: vec![IcNodeProviderRewardHistoryObservation {
                    timestamp_unix_secs: 1_784_073_600,
                    amount_e8s: 66_852_931_445_000,
                }],
            })
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
