use super::{
    CloudEngineProviderInfoReport, CloudEngineProviderListReport, CloudEngineProviderLocation,
    CloudEngineProviderRow, cloud_engine_provider_info_report_text,
    cloud_engine_provider_list_report_text,
};
use crate::ic::IcDashboardReportProvenance;

const PROVIDER_A: &str = "2wxzd-qrbrs-ailta-kdtyb-ucg35-xcxd4-txevb-ot7hx-wiyus-szcca-nqe";
const PROVIDER_B: &str = "rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae";

#[test]
fn text_keeps_dashboard_provenance_separate_from_provider_tables() {
    let provider = provider(PROVIDER_B, true);
    let list = CloudEngineProviderListReport {
        provenance: provenance(),
        source_node_provider_count: 2,
        cloud_engine_provider_count: 1,
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

#[cfg(feature = "dashboard-host")]
mod host {
    use super::*;
    use crate::{
        cloud_engine::{
            CloudEngineProviderInfoRequest, CloudEngineProviderInfoSourceData,
            CloudEngineProviderListRequest, CloudEngineProviderListSourceData,
            CloudEngineProviderSource, DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
            build_cloud_engine_provider_info_report_with_source,
            build_cloud_engine_provider_list_report_with_source,
        },
        ic::{IcHostError, IcSourceRequest},
    };
    use std::cell::Cell;

    const NOW: u64 = 1_800_000_000;

    #[test]
    fn list_validates_complete_resource_then_filters_and_sorts_cloud_engine_rows() {
        let source = Fixture::default();
        let report = build_cloud_engine_provider_list_report_with_source(&list_request(), &source)
            .expect("CloudEngine providers");

        assert_eq!(source.list_calls.get(), 1);
        assert_eq!(source.info_calls.get(), 0);
        assert_eq!(report.source_node_provider_count, 3);
        assert_eq!(report.cloud_engine_provider_count, 2);
        assert_eq!(report.providers[0].principal_id, PROVIDER_A);
        assert_eq!(report.providers[1].principal_id, PROVIDER_B);
        assert_eq!(report.provenance.authority, "official_ic_dashboard_api");
        assert!(!report.provenance.certified);
        assert!(!report.provenance.point_in_time_guaranteed);
    }

    #[test]
    fn exact_info_preserves_zero_cloud_engine_evidence_without_hiding_the_provider() {
        let source = Fixture::default();
        let request = CloudEngineProviderInfoRequest::new(
            "ic",
            DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
            NOW,
            PROVIDER_A,
        );
        source.info_cloud_engine.set(false);
        let report = build_cloud_engine_provider_info_report_with_source(&request, &source)
            .expect("exact provider");

        assert_eq!(source.info_calls.get(), 1);
        assert!(!report.cloud_engine_evidence_present);
        assert_eq!(report.provider.principal_id, PROVIDER_A);
    }

    #[test]
    fn cloud_engine_and_ordinary_location_scopes_are_independent() {
        let source = Fixture::default();
        source
            .mutation
            .set(Some(Mutation::IndependentCloudLocation));
        let report = build_cloud_engine_provider_list_report_with_source(&list_request(), &source)
            .expect("independent CloudEngine provider location");

        assert_eq!(report.providers[1].cloud_engine_locations[0].dc_key, "ce1");
        assert_eq!(report.providers[1].locations[0].dc_key, "br1");
    }

    #[test]
    fn requests_and_custom_source_contract_fail_closed() {
        let source = Fixture::default();
        let unsupported = CloudEngineProviderListRequest::new(
            "local",
            DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
            NOW,
        );
        assert!(matches!(
            build_cloud_engine_provider_list_report_with_source(&unsupported, &source),
            Err(IcHostError::InvalidRequest {
                field: "network",
                ..
            })
        ));
        assert_eq!(source.list_calls.get(), 0);

        for mutation in [
            Mutation::WrongSource,
            Mutation::Empty,
            Mutation::DuplicateProvider,
            Mutation::BadLocationCount,
            Mutation::UnassignedOverflow,
        ] {
            source.mutation.set(Some(mutation));
            assert!(matches!(
                build_cloud_engine_provider_list_report_with_source(&list_request(), &source),
                Err(IcHostError::InvalidSourceData { .. })
            ));
        }
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        WrongSource,
        Empty,
        DuplicateProvider,
        BadLocationCount,
        IndependentCloudLocation,
        UnassignedOverflow,
    }

    #[derive(Default)]
    struct Fixture {
        list_calls: Cell<usize>,
        info_calls: Cell<usize>,
        info_cloud_engine: Cell<bool>,
        mutation: Cell<Option<Mutation>>,
    }

    impl CloudEngineProviderSource for Fixture {
        fn fetch_cloud_engine_provider_list(
            &self,
            request: &IcSourceRequest,
        ) -> Result<CloudEngineProviderListSourceData, IcHostError> {
            self.list_calls.set(self.list_calls.get() + 1);
            let mut data = CloudEngineProviderListSourceData {
                source: request.clone(),
                providers: vec![
                    provider(PROVIDER_B, true),
                    provider(PROVIDER_A, true),
                    provider("aaaaa-aa", false),
                ],
            };
            match self.mutation.take() {
                Some(Mutation::WrongSource) => data.source.endpoint.push_str("/other"),
                Some(Mutation::Empty) => data.providers.clear(),
                Some(Mutation::DuplicateProvider) => {
                    data.providers[1].principal_id = data.providers[0].principal_id.clone();
                }
                Some(Mutation::BadLocationCount) => data.providers[0].location_count = 2,
                Some(Mutation::IndependentCloudLocation) => {
                    data.providers[0].cloud_engine_locations[0].dc_key = "ce1".to_string();
                    data.providers[0].cloud_engine_locations[0].owner = "Owner ".to_string();
                }
                Some(Mutation::UnassignedOverflow) => {
                    data.providers[0].total_cloud_engine_unassigned_nodes = 6;
                }
                None => {}
            }
            Ok(data)
        }

        fn fetch_cloud_engine_provider_info(
            &self,
            request: &IcSourceRequest,
            node_provider_id: &str,
        ) -> Result<CloudEngineProviderInfoSourceData, IcHostError> {
            self.info_calls.set(self.info_calls.get() + 1);
            Ok(CloudEngineProviderInfoSourceData {
                source: request.clone(),
                provider: provider(node_provider_id, self.info_cloud_engine.get()),
            })
        }
    }

    fn list_request() -> CloudEngineProviderListRequest {
        CloudEngineProviderListRequest::new(
            "ic",
            DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
            NOW,
        )
    }
}
