use crate::cloud_engine::cloud_engine_provider_info_report_text;
use crate::cloud_engine::cloud_engine_provider_list_report_text;
use ic_query::cloud_engine::CloudEngineProviderInfoReport;
use ic_query::cloud_engine::CloudEngineProviderListReport;
use ic_query::cloud_engine::CloudEngineProviderLocation;
use ic_query::cloud_engine::CloudEngineProviderRow;
use ic_query::ic::IcDashboardReportProvenance;

const PROVIDER_B: &str = "rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae";

#[test]
fn text_keeps_dashboard_provenance_separate_from_provider_tables() {
    let provider = provider(PROVIDER_B, true);
    let list = CloudEngineProviderListReport {
        provenance: provenance(),
        source_node_provider_count: 2,
        providers: vec![provider.clone()],
    };
    let info = CloudEngineProviderInfoReport {
        provenance: provenance(),
        cloud_engine_evidence_present: true,
        provider,
    };

    let list_text = cloud_engine_provider_list_report_text(&list);
    let info_text = cloud_engine_provider_info_report_text(&info);

    assert!(list_text.contains("point_in_time_guaranteed: no"));
    assert!(list_text.contains("cloud_engine_provider_count: 1\n\nCloudEngine providers\n"));
    assert!(info_text.contains("cloud_engine_evidence_present: yes"));
    assert!(info_text.contains("location_count: 1\n\nCloudEngine locations\n"));
    assert!(info_text.contains("Brussels"));
}

fn provenance() -> IcDashboardReportProvenance {
    IcDashboardReportProvenance {
        schema_version: 1,
        network: "ic".to_string(),
        authority: "official_ic_dashboard_api".to_string(),
        source_endpoint: "https://ic-api.internetcomputer.org/api/v3".to_string(),
        fetched_at: "2026-08-08T12:00:00Z".to_string(),
        fetched_by: "test".to_string(),
        certified: false,
        point_in_time_guaranteed: false,
    }
}

fn location() -> CloudEngineProviderLocation {
    CloudEngineProviderLocation {
        dc_key: "br1".to_string(),
        display_name: "Brussels".to_string(),
        latitude: 50.8386,
        longitude: 4.3475,
        owner: "Digital Realty".to_string(),
        region: "Europe,BE,Brussels Capital".to_string(),
    }
}

fn provider(principal_id: &str, cloud_engine: bool) -> CloudEngineProviderRow {
    let locations = vec![location()];
    CloudEngineProviderRow {
        principal_id: principal_id.to_string(),
        display_name: "Provider".to_string(),
        website: Some("example.com".to_string()),
        logo_url: None,
        location_count: locations.len(),
        locations: locations.clone(),
        cloud_engine_location_count: usize::from(cloud_engine),
        cloud_engine_locations: cloud_engine
            .then_some(locations[0].clone())
            .into_iter()
            .collect(),
        total_cloud_engine_nodes: if cloud_engine { 5 } else { 0 },
        total_cloud_engine_unassigned_nodes: if cloud_engine { 4 } else { 0 },
        total_cloud_engines: u64::from(cloud_engine),
        total_node_allowance: 7,
        total_nodes: 8,
        total_rewardable_nodes: 6,
        total_subnets: 2,
        total_unassigned_nodes: 3,
    }
}
