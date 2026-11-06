use super::NnsDataCenterListReport;
use super::NnsDataCenterRow;
use super::NnsInventoryError;
use super::NnsInventoryListRequest;
use super::build_nns_data_center_list_report_with_source_and_progress;
use super::resolve_data_center;
use crate::nns::LiveNnsSource;
use crate::nns::NnsInventoryCacheRequest;
use crate::nns::NnsRegistryInventorySource;
use crate::report::CollectionSourceRequest;
use crate::subnet_catalog::MAINNET_NETWORK;
use crate::subnet_catalog::MAINNET_REGISTRY_CANISTER_ID;
use crate::tests::temp_dir;

#[tokio::test(flavor = "current_thread")]
async fn live_data_center_source_rejects_non_mainnet_before_agent_construction() {
    let request = CollectionSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = LiveNnsSource::default()
        .fetch_registry_inventory(&request)
        .await
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsInventoryError::UnsupportedNetwork { collection: crate::nns::NnsInventoryCollection::RegistryInventory, network } if network == "local"
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn data_center_report_uses_live_registry_source() {
    let request = NnsInventoryListRequest {
        cache: test_cache_request(MAINNET_NETWORK, "uses-live-source"),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
    };
    let report = build_nns_data_center_list_report_with_source_and_progress(
        &request,
        &FixtureDataCenterSource {
            data_centers: vec![data_center_fixture()],
        },
        &mut |_| {},
    )
    .await
    .expect("data-center report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.provenance.network, MAINNET_NETWORK);
    assert_eq!(
        report.provenance.registry_canister_id,
        MAINNET_REGISTRY_CANISTER_ID
    );
    assert_eq!(report.provenance.registry_version, 42);
    assert_eq!(report.provenance.fetched_at, "2026-06-04T00:00:00Z");
    assert_eq!(report.data_centers.len(), 1);
    assert_eq!(report.data_centers[0].data_center_id, "dc1");
}

#[test]
fn data_center_info_resolves_unique_prefix() {
    let report = data_center_report_fixture();

    let (index, resolved_from) =
        resolve_data_center(&report.data_centers, "dc").expect("prefix resolves");
    let data_center = &report.data_centers[index];

    assert_eq!(resolved_from, "data_center_id_prefix");
    assert_eq!(data_center.data_center_id, "dc1");
}

fn data_center_report_fixture() -> NnsDataCenterListReport {
    NnsDataCenterListReport {
        schema_version: 1,
        provenance: crate::nns::registry::NnsRegistryInventoryProvenance {
            network: MAINNET_NETWORK.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: "2026-06-04T00:00:00Z".to_string(),
            source_endpoint: "https://icp-api.io".to_string(),
            fetched_by: "test".to_string(),
        },
        data_centers: vec![NnsDataCenterRow {
            data_center_id: "dc1".to_string(),
            region: "eu-west".to_string(),
            owner: "example owner".to_string(),
            latitude: Some(48.8566),
            longitude: Some(2.3522),
            node_operator_count: 2,
            node_provider_count: 1,
            node_count: 13,
        }],
    }
}

fn data_center_fixture() -> NnsDataCenterRow {
    NnsDataCenterRow {
        data_center_id: "dc1".to_string(),
        region: "eu-west".to_string(),
        owner: "example owner".to_string(),
        latitude: Some(48.8566),
        longitude: Some(2.3522),
        node_operator_count: 2,
        node_provider_count: 1,
        node_count: 13,
    }
}

fn test_cache_request(network: &str, name: &str) -> NnsInventoryCacheRequest {
    NnsInventoryCacheRequest {
        cache_root: temp_dir(&format!("ic-query-nns-data-center-{name}")),
        network: network.to_string(),
    }
}

struct FixtureDataCenterSource {
    data_centers: Vec<NnsDataCenterRow>,
}

impl crate::nns::NnsRegistryInventorySource for FixtureDataCenterSource {
    fn fetch_registry_inventory<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<crate::nns::NnsRegistryInventorySnapshot, NnsInventoryError>,
    > {
        Box::pin(async move {
            let result: Result<NnsDataCenterListReport, NnsInventoryError> = async move {
                let data_centers = self.data_centers.clone();
                Ok(NnsDataCenterListReport {
                    schema_version: 1,
                    provenance: crate::nns::registry::NnsRegistryInventoryProvenance {
                        network: MAINNET_NETWORK.to_string(),
                        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
                        registry_version: 42,
                        fetched_at: request.fetched_at.clone(),
                        source_endpoint: request.endpoint.clone(),
                        fetched_by: request.fetched_by.clone(),
                    },
                    data_centers,
                })
            }
            .await;
            result.map(Into::into)
        })
    }
}
