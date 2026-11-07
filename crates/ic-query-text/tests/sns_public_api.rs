//! Downstream neuron ownership and raw stake evidence.
#[cfg(feature = "sns-host")]
mod support;

use ic_query::sns::{
    SnsInstanceContext, SnsNeuronDetailReport, SnsNeuronDissolveState, SnsNeuronPermissionRow,
    SnsNeuronRow, SnsReportContext,
};
const GOV: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
fn neuron() -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: "01".repeat(32),
        permissions: vec![SnsNeuronPermissionRow {
            principal: "aaaaa-aa".to_string(),
            permission_types: vec![5, 999],
        }],
        cached_neuron_stake_e8s: 125_000_000,
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
#[test]
fn exact_neuron_json_and_text_preserve_permission_codes_and_raw_stake() {
    let report = SnsNeuronDetailReport {
        schema_version: 1,
        context: SnsReportContext {
            instance: SnsInstanceContext {
                network: "ic".to_string(),
                fetched_at: "2026-06-04T00:00:00Z".to_string(),
                source_endpoint: "https://icp-api.io".to_string(),
                fetched_by: "fixture".to_string(),
            },
            governance_canister_id: GOV.to_string(),
        },
        neuron_id: neuron().neuron_id,
        data_source: ic_query::report::ReportDataSource::Live,
        neuron: neuron(),
    };
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["neuron"]["cached_neuron_stake_e8s"], 125_000_000);
    assert_eq!(
        json["neuron"]["permissions"][0]["permission_types"],
        serde_json::json!([5, 999])
    );
    assert_eq!(json["governance_canister_id"], GOV);
    let decoded: SnsNeuronDetailReport = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, report);
    let text = ic_query_text::sns::sns_neuron_detail_report_text(&report);
    assert!(text.contains(GOV));
    assert!(text.contains("5:disburse"));
    assert!(text.contains("999:unknown"));
}
#[cfg(feature = "sns-host")]
mod host {
    use super::*;
    use ic_query::{
        report::{CollectionSourceRequest, SourceFuture},
        sns::*,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Default)]
    struct Source(AtomicUsize);
    impl SnsNeuronsSource for Source {
        fn fetch_sns_neurons<'a>(
            &'a self,
            request: &'a CollectionSourceRequest,
            governance: &'a str,
            _: u32,
            owner: Option<&'a str>,
        ) -> SourceFuture<'a, Result<Vec<SnsNeuronRow>, SnsHostError>> {
            Box::pin(async move {
                assert_eq!(request.network, "ic");
                assert_eq!(governance, GOV);
                assert_eq!(owner, Some("aaaaa-aa"));
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(vec![neuron()])
            })
        }
        fn fetch_sns_neuron_page<'a>(
            &'a self,
            _: &'a CollectionSourceRequest,
            governance: &'a str,
            _: u32,
            start: Option<&'a SnsNeuronId>,
            owner: Option<&'a str>,
        ) -> SourceFuture<'a, Result<MainnetSnsNeuronPage, SnsHostError>> {
            Box::pin(async move {
                assert_eq!(governance, GOV);
                assert!(owner.is_none());
                self.0.fetch_add(1, Ordering::SeqCst);
                assert!(start.is_none());
                Ok(MainnetSnsNeuronPage {
                    neurons: vec![neuron()],
                    last_cursor: Some(SnsNeuronId { id: vec![1; 32] }),
                })
            })
        }
    }
    #[tokio::test(flavor = "current_thread")]
    async fn downstream_sources_and_complete_cache_need_only_governance() {
        let source = Source::default();
        let target =
            SnsNeuronTargetRequest::new("ic", "https://icp-api.io", 1_780_531_200, GOV).unwrap();
        let live = build_sns_neurons_report_with_source(
            &SnsNeuronsRequest::live(target.clone(), 2, Some("aaaaa-aa".to_string())),
            &source,
        )
        .await
        .unwrap();
        assert_eq!(live.neurons, [neuron()]);
        let root = crate::support::temp_root("ic-query-neuron-api");
        let refresh = refresh_sns_neurons_cache_with_source_and_progress(
            &SnsSnapshotRefreshRequest::new(&root, target.clone(), 2),
            &source,
            &mut |_| {},
        )
        .await
        .unwrap();
        assert_eq!(refresh.row_count, 1);
        let calls = source.0.load(Ordering::SeqCst);
        let cached = build_sns_neurons_report_with_source(
            &SnsNeuronsRequest::cached(target, 10, &root, SnsNeuronsCacheSort::Stake),
            &source,
        )
        .await
        .unwrap();
        assert_eq!(cached.neurons, [neuron()]);
        assert_eq!(source.0.load(Ordering::SeqCst), calls);
        assert!(
            ic_query_text::sns::sns_neurons_report_text(
                &cached,
                ic_query_text::TextOptions::default()
            )
            .contains("1.25")
        );
        let status =
            build_sns_neurons_cache_status_report(&SnsCacheStatusRequest::new(&root, "ic", GOV))
                .unwrap();
        assert!(status.found);
        assert!(ic_query_text::sns::sns_neurons_cache_status_report_text(&status).contains(GOV));
    }
}
