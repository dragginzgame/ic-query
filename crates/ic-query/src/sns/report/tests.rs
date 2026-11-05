use super::*;
use crate::report::{CollectionSourceRequest, SourceFuture};
use std::sync::Mutex;
const GOV: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
const OWNER: &str = "aaaaa-aa";
fn row(id: u8, stake: u64) -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: crate::hex::hex_bytes(&[id; 32]),
        permissions: vec![SnsNeuronPermissionRow {
            principal: OWNER.to_string(),
            permission_types: vec![5, 999],
        }],
        cached_neuron_stake_e8s: stake,
        neuron_fees_e8s: 10,
        maturity_e8s_equivalent: 2,
        staked_maturity_e8s_equivalent: Some(3),
        created_timestamp_seconds: 4,
        source_nns_neuron_id: None,
        auto_stake_maturity: Some(false),
        aging_since_timestamp_seconds: 5,
        dissolve_state: Some(SnsNeuronDissolveState::DissolveDelaySeconds(60)),
        voting_power_percentage_multiplier: 100,
        vesting_period_seconds: None,
    }
}
fn target() -> SnsNeuronTargetRequest {
    SnsNeuronTargetRequest::new("ic", "https://icp-api.io", 1_780_531_200, GOV).unwrap()
}
#[derive(Default)]
struct Source {
    calls: Mutex<Vec<String>>,
    fail: bool,
    wrong_owner: bool,
}
impl SnsNeuronsSource for Source {
    fn fetch_sns_neurons<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        governance: &'a str,
        limit: u32,
        owner: Option<&'a str>,
    ) -> SourceFuture<'a, Result<Vec<SnsNeuronRow>, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(request.endpoint, "https://icp-api.io");
            assert_eq!(governance, GOV);
            assert_eq!(limit, 2);
            assert_eq!(owner, Some(OWNER));
            self.calls.lock().unwrap().push(governance.to_string());
            let mut neuron = row(1, 100);
            if self.wrong_owner {
                neuron.permissions.clear();
            }
            Ok(vec![neuron])
        })
    }
    fn fetch_sns_neuron_page<'a>(
        &'a self,
        _: &'a CollectionSourceRequest,
        governance: &'a str,
        limit: u32,
        start: Option<&'a SnsNeuronId>,
        owner: Option<&'a str>,
    ) -> SourceFuture<'a, Result<MainnetSnsNeuronPage, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(governance, GOV);
            assert_eq!(limit, 2);
            assert!(owner.is_none());
            self.calls.lock().unwrap().push(governance.to_string());
            if self.fail && start.is_some() {
                return Err(SnsHostError::AgentCall {
                    method: "list_neurons",
                    reason: "fixture unavailable".to_string(),
                });
            }
            let neurons = match start.map(|id| id.id[0]) {
                None => vec![row(1, 100), row(2, 300)],
                Some(2) => vec![row(2, 999), row(3, 200)],
                _ => Vec::new(),
            };
            let last_cursor = neurons.last().map(|n| SnsNeuronId {
                id: crate::hex::decode_lowercase_hex(&n.neuron_id).unwrap(),
            });
            Ok(MainnetSnsNeuronPage {
                neurons,
                last_cursor,
            })
        })
    }
}
impl SnsNeuronSource for Source {
    fn fetch_sns_neuron<'a>(
        &'a self,
        _: &'a CollectionSourceRequest,
        governance: &'a str,
        id: &'a str,
    ) -> SourceFuture<'a, Result<SnsNeuronRow, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(governance, GOV);
            let neuron = row(1, 100);
            assert_eq!(id, neuron.neuron_id);
            Ok(neuron)
        })
    }
}
#[test]
fn explicit_targets_reject_numeric_ids_and_path_inputs() {
    for bad in ["1", "../cache", "", "https://example.com"] {
        assert!(SnsNeuronTargetRequest::new("ic", "https://icp-api.io", 0, bad).is_err());
    }
}
#[tokio::test(flavor = "current_thread")]
async fn direct_reads_preserve_stake_permission_codes_and_target() {
    let source = Source::default();
    let request = SnsNeuronsRequest::live(target(), 2, Some(OWNER.to_string()));
    let report = build_sns_neurons_report_with_source(&request, &source)
        .await
        .unwrap();
    assert_eq!(report.context.governance_canister_id, GOV);
    assert_eq!(report.neurons[0], row(1, 100));
    assert_eq!(report.neurons[0].permissions[0].permission_types, [5, 999]);
    let exact = build_sns_neuron_detail_report_with_source(
        &SnsNeuronRequest::new(target(), row(1, 0).neuron_id),
        &source,
    )
    .await
    .unwrap();
    assert_eq!(exact.neuron, report.neurons[0]);
    assert!(matches!(
        build_sns_neurons_report_with_source(
            &request,
            &Source {
                wrong_owner: true,
                ..Default::default()
            }
        )
        .await,
        Err(SnsHostError::InvalidSourceData { .. })
    ));
}
#[tokio::test(flavor = "current_thread")]
async fn complete_snapshots_keep_permissions_and_repeated_boundaries_once() {
    let root = crate::tests::temp_dir("explicit-governance-neuron-snapshot");
    let source = Source::default();
    let request = SnsSnapshotRefreshRequest::new(&root, target(), 2);
    let refresh =
        refresh_sns_neurons_cache_with_source_and_progress(&request, &source, &mut |_| {})
            .await
            .unwrap();
    assert_eq!(refresh.row_count, 3);
    assert_eq!(refresh.page_count, 3);
    assert!(refresh.cache_path.contains("sns-neuron/ic/"));
    let before = std::fs::read(&refresh.cache_path).unwrap();
    let read = SnsNeuronsRequest::cached(target(), 10, &root, SnsNeuronsCacheSort::Stake);
    let report = build_sns_neurons_report_with_source(
        &read,
        &Source {
            fail: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        report
            .neurons
            .iter()
            .map(|n| n.cached_neuron_stake_e8s)
            .collect::<Vec<_>>(),
        [300, 200, 100]
    );
    assert_eq!(report.neurons[0].permissions, row(2, 0).permissions);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&before).unwrap()["completeness"]["point_in_time_guaranteed"],
        false
    );
    assert!(
        refresh_sns_neurons_cache_with_source_and_progress(
            &request,
            &Source {
                fail: true,
                ..Default::default()
            },
            &mut |_| {}
        )
        .await
        .is_err()
    );
    assert_eq!(std::fs::read(&refresh.cache_path).unwrap(), before);
    let status =
        build_sns_neurons_cache_status_report(&SnsCacheStatusRequest::new(&root, "ic", GOV))
            .unwrap();
    assert!(status.found);
    assert_eq!(
        status
            .cache
            .unwrap()
            .valid()
            .unwrap()
            .governance_canister_id,
        GOV
    );
    assert_eq!(
        status.latest_attempt.unwrap().lifecycle.status,
        crate::cache::CacheRefreshAttemptStatus::Failed
    );
    let mut damaged: serde_json::Value = serde_json::from_slice(&before).unwrap();
    damaged["metadata"]["governance_canister_id"] = "aaaaa-aa".into();
    std::fs::write(&refresh.cache_path, serde_json::to_vec(&damaged).unwrap()).unwrap();
    let invalid =
        build_sns_neurons_cache_status_report(&SnsCacheStatusRequest::new(&root, "ic", GOV))
            .unwrap();
    assert!(invalid.cache.unwrap().valid().is_none());
}
#[tokio::test(flavor = "current_thread")]
async fn incomplete_refresh_preserves_previous_snapshot() {
    let root = crate::tests::temp_dir("explicit-neuron-page-ceiling");
    let source = Source::default();
    let mut request = SnsSnapshotRefreshRequest::new(&root, target(), 2);
    let initial =
        refresh_sns_neurons_cache_with_source_and_progress(&request, &source, &mut |_| {})
            .await
            .unwrap();
    let bytes = std::fs::read(&initial.cache_path).unwrap();
    request.max_pages = Some(1);
    assert!(matches!(
        refresh_sns_neurons_cache_with_source_and_progress(&request, &source, &mut |_| {}).await,
        Err(SnsHostError::IncompleteRefresh { .. })
    ));
    assert_eq!(std::fs::read(&initial.cache_path).unwrap(), bytes);
}
