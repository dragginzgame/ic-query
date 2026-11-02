//! Replay report values and exact output captured before the crate extraction.
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
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/renderer_contracts.json")).unwrap();
    for fixture in fixtures {
        let renderer = fixture["renderer"].as_str().unwrap();
        let report = &fixture["report"];
        let options = ic_query_text::TextOptions {
            verbose: fixture["options"]["verbose"].as_bool().unwrap_or(false),
            show_ballots: fixture["options"]["show_ballots"]
                .as_bool()
                .unwrap_or(false),
        };
        let actual = render_report(renderer, report, options);
        assert_eq!(actual, fixture["text"].as_str().unwrap(), "{renderer}");
    }
}

fn render_report(
    renderer: &str,
    report: &serde_json::Value,
    options: ic_query_text::TextOptions,
) -> String {
    match renderer {
        "cloud_engine_list_report_text" => render(cloud_engine_list_report_text, report),
        "cloud_engine_operator_report_text" => render(cloud_engine_operator_report_text, report),
        "cloud_engine_prices_report_text" => render(cloud_engine_prices_report_text, report),
        "cmc_cycles_report_text" => render(cmc::cmc_cycles_report_text, report),
        "cmc_xdr_report_text" => render(cmc::cmc_xdr_report_text, report),
        "nns_data_center_list_report_text" => {
            render(data_center::nns_data_center_list_report_text, report)
        }
        "nns_data_center_list_report_verbose_text" => render(
            data_center::nns_data_center_list_report_verbose_text,
            report,
        ),
        "nns_neuron_distribution_report_text" => {
            render(neuron::nns_neuron_distribution_report_text, report)
        }
        "nns_node_list_report_text" => render(node::nns_node_list_report_text, report),
        "nns_node_list_report_verbose_text" => {
            render(node::nns_node_list_report_verbose_text, report)
        }
        "nns_node_operator_list_report_text" => {
            render(node_operator::nns_node_operator_list_report_text, report)
        }
        "nns_node_operator_list_report_verbose_text" => render(
            node_operator::nns_node_operator_list_report_verbose_text,
            report,
        ),
        "nns_node_provider_info_report_text" => {
            render(node_provider::nns_node_provider_info_report_text, report)
        }
        "nns_node_provider_list_report_text" => {
            render(node_provider::nns_node_provider_list_report_text, report)
        }
        "nns_node_provider_list_report_verbose_text" => render(
            node_provider::nns_node_provider_list_report_verbose_text,
            report,
        ),
        "nns_proposal_activity_report_text" => {
            render(proposals::nns_proposal_activity_report_text, report)
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
        "sns_reward_checkpoint_report_text" => {
            render(sns::sns_reward_checkpoint_report_text, report)
        }
        "subnet_catalog_info_report_text" => {
            render(subnet_catalog::subnet_catalog_info_report_text, report)
        }
        "subnet_catalog_list_report_text" => {
            render(subnet_catalog::subnet_catalog_list_report_text, report)
        }
        "subnet_catalog_list_report_verbose_text" => render(
            subnet_catalog::subnet_catalog_list_report_verbose_text,
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
