use super::*;
use crate::{
    nns::{LiveNnsSource, NnsSourceRequest},
    subnet_catalog::MAINNET_NETWORK,
};
use wire::{
    GetMaturityModulationResponse, GetMetricsResult, GovernanceCachedMetrics, GovernanceError,
    NeuronSubsetMetrics,
};

#[test]
fn governance_report_context_serializes_flattened() {
    let report = NnsGovernanceMaturityModulationReport {
        context: NnsGovernanceReportContext {
            schema_version: 1,
            network: "ic".to_string(),
            governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
            fetched_at: "2026-07-30T00:00:00Z".to_string(),
            source_endpoint: DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT.to_string(),
            fetched_by: "fixture".to_string(),
        },
        maturity_modulation: None,
    };

    let value = serde_json::to_value(report).expect("serialize report");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["network"], "ic");
    assert!(value.get("context").is_none());
    assert!(value["maturity_modulation"].is_null());
}

#[test]
fn live_governance_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsSourceRequest::new(
        "local",
        "this-is-not-a-valid-url",
        "2026-07-30T00:00:00Z",
        "fixture",
    );

    assert!(matches!(
        LiveNnsSource.fetch_economics(&request),
        Err(NnsGovernanceHostError::UnsupportedNetwork { network }) if network == "local"
    ));
    assert!(matches!(
        LiveNnsSource.fetch_metrics(&request),
        Err(NnsGovernanceHostError::UnsupportedNetwork { network }) if network == "local"
    ));
    assert!(matches!(
        LiveNnsSource.fetch_reward_event(&request),
        Err(NnsGovernanceHostError::UnsupportedNetwork { network }) if network == "local"
    ));
    assert!(matches!(
        LiveNnsSource.fetch_maturity_modulation(&request),
        Err(NnsGovernanceHostError::UnsupportedNetwork { network }) if network == "local"
    ));
}

#[test]
fn custom_source_builder_preserves_shared_provenance() {
    let request = NnsSourceRequest::new(
        MAINNET_NETWORK,
        "https://example.test",
        "2026-07-30T00:00:00Z",
        "fixture",
    );

    let report = build_nns_governance_economics_report_with_source(&request, &FixtureSource)
        .expect("fixture economics report");

    assert_eq!(report.context.network, MAINNET_NETWORK);
    assert_eq!(report.context.source_endpoint, "https://example.test");
    assert_eq!(report.context.fetched_by, "fixture");
    assert_eq!(report.economics.reject_cost_e8s, 1_000_000_000);
}

#[test]
fn builder_rejects_non_mainnet_before_custom_source_call() {
    let request = NnsSourceRequest::new(
        "local",
        "https://example.test",
        "2026-07-30T00:00:00Z",
        "fixture",
    );

    assert!(matches!(
        build_nns_governance_economics_report_with_source(&request, &PanicSource),
        Err(NnsGovernanceHostError::UnsupportedNetwork { network }) if network == "local"
    ));
}

#[test]
fn economics_candid_shape_preserves_nested_native_options() {
    let economics = NnsGovernanceEconomics {
        neuron_minimum_stake_e8s: 100_000_000,
        max_proposals_to_keep_per_topic: 100,
        neuron_management_fee_per_proposal_e8s: 10_000,
        reject_cost_e8s: 1_000_000_000,
        transaction_fee_e8s: 10_000,
        neuron_spawn_dissolve_delay_seconds: 604_800,
        minimum_icp_xdr_rate: 100,
        maximum_node_provider_rewards_e8s: 200_000_000,
        neurons_fund_economics: Some(NnsNeuronsFundEconomics {
            maximum_icp_xdr_rate: Some(NnsGovernancePercentage { basis_points: None }),
            neurons_fund_matched_funding_curve_coefficients: Some(
                NnsNeuronsFundMatchedFundingCurveCoefficients {
                    contribution_threshold_xdr: Some(NnsGovernanceDecimal {
                        human_readable: Some("1.25".to_string()),
                    }),
                    one_third_participation_milestone_xdr: None,
                    full_participation_milestone_xdr: None,
                },
            ),
            max_theoretical_neurons_fund_participation_amount_xdr: None,
            minimum_icp_xdr_rate: None,
        }),
        voting_power_economics: Some(NnsVotingPowerEconomics {
            start_reducing_voting_power_after_seconds: Some(15_778_800),
            clear_following_after_seconds: Some(2_629_800),
            neuron_minimum_dissolve_delay_to_vote_seconds: Some(15_778_800),
        }),
    };
    let bytes = candid::encode_one(&economics).expect("encode native economics");
    let economics: NnsGovernanceEconomics =
        candid::decode_one(&bytes).expect("decode native economics");

    let fund = economics
        .neurons_fund_economics
        .expect("Neurons' Fund economics");
    assert_eq!(
        fund.maximum_icp_xdr_rate
            .expect("maximum rate wrapper")
            .basis_points,
        None
    );
    assert_eq!(
        fund.neurons_fund_matched_funding_curve_coefficients
            .expect("curve")
            .contribution_threshold_xdr
            .expect("threshold")
            .human_readable
            .as_deref(),
        Some("1.25")
    );
    assert_eq!(
        economics
            .voting_power_economics
            .expect("voting economics")
            .neuron_minimum_dissolve_delay_to_vote_seconds,
        Some(15_778_800)
    );
}

#[test]
fn reward_and_maturity_public_types_preserve_native_candid_shapes() {
    let reward_event = NnsGovernanceRewardEvent {
        rounds_since_last_distribution: Some(1),
        day_after_genesis: 2,
        actual_timestamp_seconds: 3,
        total_available_e8s_equivalent: 4,
        latest_round_available_e8s_equivalent: Some(5),
        distributed_e8s_equivalent: 6,
        settled_proposals: vec![NnsGovernanceProposalId { id: 7 }],
    };
    let bytes = candid::encode_one(&reward_event).expect("encode reward event");
    let decoded: NnsGovernanceRewardEvent =
        candid::decode_one(&bytes).expect("decode reward event");
    assert_eq!(decoded, reward_event);

    let response = GetMaturityModulationResponse {
        maturity_modulation: Some(NnsGovernanceMaturityModulation {
            current_value_permyriad: Some(-125),
            updated_at_timestamp_seconds: Some(8),
        }),
    };
    let bytes = candid::encode_one(&response).expect("encode maturity response");
    let decoded: GetMaturityModulationResponse =
        candid::decode_one(&bytes).expect("decode maturity response");
    let modulation = decoded.maturity_modulation.expect("maturity modulation");
    assert_eq!(modulation.current_value_permyriad, Some(-125));
    assert_eq!(modulation.updated_at_timestamp_seconds, Some(8));
}

#[test]
fn metrics_projection_names_unlabeled_buckets_and_preserves_subsets() {
    let metrics = NnsGovernanceMetrics::from(GovernanceCachedMetrics {
        total_supply_icp: 42,
        not_dissolving_neurons_e8s_buckets: vec![(10, 20.5)],
        public_neuron_subset_metrics: Some(NeuronSubsetMetrics {
            count: Some(3),
            count_buckets: vec![(100, 2)],
            ..NeuronSubsetMetrics::default()
        }),
        ..GovernanceCachedMetrics::default()
    });

    assert_eq!(metrics.total_supply_icp, 42);
    assert_eq!(
        metrics.not_dissolving_neurons_e8s_buckets,
        vec![NnsGovernanceMetricBucket {
            key: 10,
            value: 20.5,
        }]
    );
    let public = metrics.public_neuron_subset_metrics.expect("public subset");
    assert_eq!(public.count, Some(3));
    assert_eq!(
        public.count_buckets,
        vec![NnsGovernanceMetricBucket { key: 100, value: 2 }]
    );
}

#[test]
fn boxed_metrics_result_preserves_the_native_candid_variant_shape() {
    let result = GetMetricsResult::Ok(Box::new(GovernanceCachedMetrics {
        total_supply_icp: 42,
        ..GovernanceCachedMetrics::default()
    }));
    let bytes = candid::encode_one(&result).expect("encode metrics result");
    let decoded: GetMetricsResult = candid::decode_one(&bytes).expect("decode metrics result");

    let GetMetricsResult::Ok(metrics) = decoded else {
        panic!("expected native Ok metrics variant");
    };
    assert_eq!(metrics.total_supply_icp, 42);
}

#[test]
fn metrics_validation_rejects_non_finite_json_values() {
    let metrics = NnsGovernanceMetrics::from(GovernanceCachedMetrics {
        dissolving_neurons_e8s_buckets: vec![(30, f64::NAN)],
        ..GovernanceCachedMetrics::default()
    });

    let error = validate_governance_metrics(&metrics).expect_err("non-finite metric must fail");
    assert!(matches!(
        error,
        NnsGovernanceHostError::InvalidMetrics {
            field: "dissolving_neurons_e8s_buckets",
            key: 30,
            value,
        } if value.is_nan()
    ));
}

#[test]
fn metrics_governance_error_remains_typed() {
    let result = source::metrics_result(GetMetricsResult::Err(GovernanceError {
        error_type: 5,
        error_message: "unavailable".to_string(),
    }));
    let Err(error) = result else {
        panic!("typed Governance error must fail");
    };

    assert!(matches!(
        error,
        NnsGovernanceHostError::Governance {
            error_type: 5,
            message,
        } if message == "unavailable"
    ));
}

struct FixtureSource;

impl NnsGovernanceSource for FixtureSource {
    fn fetch_economics(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceEconomics, NnsGovernanceHostError> {
        Ok(sample_economics())
    }

    fn fetch_metrics(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceMetrics, NnsGovernanceHostError> {
        unreachable!("not used by this fixture")
    }

    fn fetch_reward_event(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceRewardEvent, NnsGovernanceHostError> {
        unreachable!("not used by this fixture")
    }

    fn fetch_maturity_modulation(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<Option<NnsGovernanceMaturityModulation>, NnsGovernanceHostError> {
        unreachable!("not used by this fixture")
    }
}

struct PanicSource;

impl NnsGovernanceSource for PanicSource {
    fn fetch_economics(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceEconomics, NnsGovernanceHostError> {
        panic!("source must not be called")
    }

    fn fetch_metrics(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceMetrics, NnsGovernanceHostError> {
        panic!("source must not be called")
    }

    fn fetch_reward_event(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceRewardEvent, NnsGovernanceHostError> {
        panic!("source must not be called")
    }

    fn fetch_maturity_modulation(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<Option<NnsGovernanceMaturityModulation>, NnsGovernanceHostError> {
        panic!("source must not be called")
    }
}

const fn sample_economics() -> NnsGovernanceEconomics {
    NnsGovernanceEconomics {
        neuron_minimum_stake_e8s: 100_000_000,
        max_proposals_to_keep_per_topic: 100,
        neuron_management_fee_per_proposal_e8s: 10_000,
        reject_cost_e8s: 1_000_000_000,
        transaction_fee_e8s: 10_000,
        neuron_spawn_dissolve_delay_seconds: 604_800,
        minimum_icp_xdr_rate: 100,
        maximum_node_provider_rewards_e8s: 200_000_000,
        neurons_fund_economics: None,
        voting_power_economics: None,
    }
}
