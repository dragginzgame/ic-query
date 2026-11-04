//! Pin human-facing output independently of report construction.
#![cfg(feature = "host")]

use ic_query_text::cloud_engine::{
    cloud_engine_list_report_text, cloud_engine_operator_report_text,
    cloud_engine_prices_report_text,
};
use ic_query_text::nns::data_center;
use ic_query_text::nns::neuron;
use ic_query_text::nns::node;
use ic_query_text::nns::node_operator;
use ic_query_text::nns::node_provider;
use ic_query_text::nns::proposals;
use ic_query_text::nns::registry;
use ic_query_text::nns::topology;
use ic_query_text::sns;
use ic_query_text::subnet_catalog;
use ic_query_text::system::cmc;

#[test]
fn preserves_existing_report_text() {
    let mut fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/renderer_contracts.json")).unwrap();
    let mut changed = Vec::new();
    for fixture in &mut fixtures {
        let renderer = fixture["renderer"].as_str().unwrap();
        let options = ic_query_text::TextOptions {
            verbose: fixture["options"]["verbose"].as_bool().unwrap_or(false),
        };
        let actual = render_report(
            renderer,
            &fixture["report"],
            options,
            fixture["case"].as_str(),
        );
        if actual != fixture["text"].as_str().unwrap() {
            changed.push(format!("{renderer} verbose={}", options.verbose));
        }
        fixture["text"] = actual.into();
    }
    if let Some(path) = std::env::var_os("IC_QUERY_RENDERER_FIXTURE_OUTPUT") {
        std::fs::write(
            path,
            serde_json::to_string_pretty(&fixtures).unwrap() + "\n",
        )
        .unwrap();
    }
    assert!(
        changed.is_empty(),
        "changed renderer contracts: {changed:?}"
    );
}

fn render_report(
    renderer: &str,
    report: &serde_json::Value,
    options: ic_query_text::TextOptions,
    case: Option<&str>,
) -> String {
    match renderer {
        "cloud_engine_list_report_text" => render(cloud_engine_list_report_text, report),
        "cloud_engine_operator_report_text" => render(cloud_engine_operator_report_text, report),
        "cloud_engine_prices_report_text" => render(cloud_engine_prices_report_text, report),
        "cmc_cycles_report_text" => render(cmc::cmc_cycles_report_text, report),
        "cmc_xdr_report_text" => render(cmc::cmc_xdr_report_text, report),
        "nns_data_center_list_report_text" => render(
            |report| data_center::nns_data_center_list_report_text(report, options),
            report,
        ),
        "nns_neuron_distribution_report_text" => {
            let fixture = distribution_fixture(case.unwrap());
            assert_eq!(serde_json::to_value(&fixture).unwrap(), *report);
            neuron::nns_neuron_distribution_report_text(&fixture)
        }
        "nns_node_list_report_text" => render(
            |report| node::nns_node_list_report_text(report, options),
            report,
        ),
        "nns_node_operator_list_report_text" => render(
            |report| node_operator::nns_node_operator_list_report_text(report, options),
            report,
        ),
        "nns_node_provider_info_report_text" => {
            render(node_provider::nns_node_provider_info_report_text, report)
        }
        "nns_node_provider_list_report_text" => render(
            |report| node_provider::nns_node_provider_list_report_text(report, options),
            report,
        ),
        "nns_proposal_activity_report_text" => {
            let fixture = activity_fixture();
            assert_eq!(case, Some("activity-window-empty"));
            assert_eq!(serde_json::to_value(&fixture).unwrap(), *report);
            proposals::nns_proposal_activity_report_text(&fixture)
        }
        "nns_proposal_list_report_text" => render(
            |report| proposals::nns_proposal_list_report_text(report, options),
            report,
        ),
        "nns_proposal_report_text" => render(
            |report| proposals::nns_proposal_report_text(report, options),
            report,
        ),
        "nns_registry_version_report_text" => {
            render(registry::nns_registry_version_report_text, report)
        }
        "nns_subnet_topology_report_text" => {
            render(topology::nns_subnet_topology_report_text, report)
        }
        "nns_topology_capacity_report_text" => {
            render(topology::nns_topology_capacity_report_text, report)
        }
        "nns_topology_check_report_text" => {
            render(topology::nns_topology_check_report_text, report)
        }
        "nns_topology_coverage_report_text" => {
            render(topology::nns_topology_coverage_report_text, report)
        }
        "nns_topology_gaps_report_text" => render(topology::nns_topology_gaps_report_text, report),
        "nns_topology_providers_report_text" => {
            render(topology::nns_topology_providers_report_text, report)
        }
        "nns_topology_refresh_report_text" => {
            render(topology::nns_topology_refresh_report_text, report)
        }
        "nns_topology_regions_report_text" => {
            render(topology::nns_topology_regions_report_text, report)
        }
        "nns_topology_summary_report_text" => {
            render(topology::nns_topology_summary_report_text, report)
        }
        "nns_topology_versions_report_text" => {
            render(topology::nns_topology_versions_report_text, report)
        }
        "sns_neuron_detail_report_text" => render(sns::sns_neuron_detail_report_text, report),
        "subnet_catalog_info_report_text" => {
            render(subnet_catalog::subnet_catalog_info_report_text, report)
        }
        "subnet_catalog_list_report_text" => render(
            |report| subnet_catalog::subnet_catalog_list_report_text(report, options),
            report,
        ),
        other => panic!("unknown renderer {other}"),
    }
}

fn render<Report: serde::de::DeserializeOwned>(
    renderer: impl FnOnce(&Report) -> String,
    value: &serde_json::Value,
) -> String {
    renderer(&serde_json::from_value(value.clone()).unwrap())
}

fn analytics_context() -> ic_query::nns::governance::NnsGovernanceAnalyticsContext {
    ic_query::nns::governance::NnsGovernanceAnalyticsContext {
        schema_version: 1,
        network: "ic".to_string(),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        collection_page_count: 1,
        collection_started_at: "2026-08-15T00:00:00Z".to_string(),
        collection_updated_at: "2026-08-15T00:00:01Z".to_string(),
        source: ic_query::nns::governance::NnsGovernanceSourceProvenance::ReplicaQuery {
            endpoint: "https://icp-api.io".to_string(),
            fetched_by: "fixture".to_string(),
        },
        point_in_time_guaranteed: false,
    }
}

fn distribution_fixture(case: &str) -> ic_query::nns::neuron::NnsNeuronDistributionReport {
    let empty = ic_query::nns::neuron::NnsNeuronDistributionReport {
        context: analytics_context(),
        collected_neuron_count: 0,
        earliest_retrieved_at_timestamp_seconds: None,
        known_neuron_metadata_count: 0,
        latest_retrieved_at_timestamp_seconds: None,
        neuron_type_distribution: Vec::new(),
        neurons_fund_join_timestamp_present_count: 0,
        reported_deciding_voting_power_neuron_count: 0,
        reported_potential_voting_power_neuron_count: 0,
        reported_staked_maturity_neuron_count: 0,
        state_distribution: Vec::new(),
        total_effective_stake_e8s: 0,
        total_reported_deciding_voting_power: 0,
        total_reported_potential_voting_power: 0,
        total_reported_staked_maturity_e8s_equivalent: 0,
        unreported_deciding_voting_power_neuron_count: 0,
        unreported_potential_voting_power_neuron_count: 0,
        unreported_staked_maturity_neuron_count: 0,
        visibility_distribution: Vec::new(),
    };
    if case == "distribution-empty" {
        return empty;
    }
    assert_eq!(case, "distribution-populated");
    ic_query::nns::neuron::NnsNeuronDistributionReport {
        collected_neuron_count: 4,
        earliest_retrieved_at_timestamp_seconds: Some(100),
        known_neuron_metadata_count: 2,
        latest_retrieved_at_timestamp_seconds: Some(120),
        neuron_type_distribution: vec![
            ic_query::nns::neuron::NnsNeuronTypeDistribution {
                effective_stake_e8s: 100,
                neuron_count: 1,
                neuron_type: None,
            },
            ic_query::nns::neuron::NnsNeuronTypeDistribution {
                effective_stake_e8s: 500,
                neuron_count: 2,
                neuron_type: Some(1),
            },
            ic_query::nns::neuron::NnsNeuronTypeDistribution {
                effective_stake_e8s: 400,
                neuron_count: 1,
                neuron_type: Some(9),
            },
        ],
        neurons_fund_join_timestamp_present_count: 2,
        reported_deciding_voting_power_neuron_count: 2,
        reported_potential_voting_power_neuron_count: 2,
        reported_staked_maturity_neuron_count: 2,
        state_distribution: vec![
            ic_query::nns::neuron::NnsNeuronStateDistribution {
                effective_stake_e8s: 400,
                neuron_count: 1,
                state: -7,
            },
            ic_query::nns::neuron::NnsNeuronStateDistribution {
                effective_stake_e8s: 300,
                neuron_count: 2,
                state: 1,
            },
            ic_query::nns::neuron::NnsNeuronStateDistribution {
                effective_stake_e8s: 300,
                neuron_count: 1,
                state: 3,
            },
        ],
        total_effective_stake_e8s: 1000,
        total_reported_deciding_voting_power: 50,
        total_reported_potential_voting_power: 90,
        total_reported_staked_maturity_e8s_equivalent: 40,
        unreported_deciding_voting_power_neuron_count: 2,
        unreported_potential_voting_power_neuron_count: 2,
        unreported_staked_maturity_neuron_count: 2,
        visibility_distribution: vec![
            ic_query::nns::neuron::NnsNeuronVisibilityDistribution {
                effective_stake_e8s: 100,
                neuron_count: 1,
                visibility: None,
            },
            ic_query::nns::neuron::NnsNeuronVisibilityDistribution {
                effective_stake_e8s: 400,
                neuron_count: 1,
                visibility: Some(-8),
            },
            ic_query::nns::neuron::NnsNeuronVisibilityDistribution {
                effective_stake_e8s: 500,
                neuron_count: 2,
                visibility: Some(2),
            },
        ],
        ..empty
    }
}

fn activity_fixture() -> ic_query::nns::proposals::NnsProposalActivityReport {
    ic_query::nns::proposals::NnsProposalActivityReport {
        context: analytics_context(),
        collected_proposal_count: 1,
        day_counts: Vec::new(),
        earliest_included_proposal_timestamp_seconds: None,
        excluded_at_or_after_until_count: 0,
        excluded_before_from_count: 1,
        from_proposal_timestamp_seconds: Some(2),
        included_proposal_count: 0,
        latest_included_proposal_timestamp_seconds: None,
        reward_status_counts: Vec::new(),
        status_counts: Vec::new(),
        topic_counts: Vec::new(),
        until_proposal_timestamp_seconds: Some(3),
    }
}
