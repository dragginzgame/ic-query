use super::{
    NnsRegistryCertification, NnsRegistryHostError, NnsRegistrySource, NnsRegistryVersionRequest,
    build_nns_registry_version_report_with_source,
};
use crate::{
    nns::LiveNnsSource,
    report::CollectionSourceRequest,
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};
use ic_host_text::timestamp::format_utc_timestamp_secs;

#[tokio::test(flavor = "current_thread")]
async fn live_registry_source_rejects_non_mainnet_before_agent_construction() {
    let request = CollectionSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = LiveNnsSource::default()
        .fetch_registry_version(&request)
        .await
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsRegistryHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn registry_version_report_uses_live_source_shape() {
    let request = NnsRegistryVersionRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
    };

    let report = build_nns_registry_version_report_with_source(&request, &FixtureNnsRegistrySource)
        .await
        .expect("registry version report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.provenance.network, MAINNET_NETWORK);
    assert_eq!(
        report.provenance.registry_canister_id,
        MAINNET_REGISTRY_CANISTER_ID
    );
    assert_eq!(report.provenance.registry_version, 42);
    assert_eq!(report.provenance.fetched_at, "2026-06-04T00:00:00Z");
    assert_eq!(report.provenance.source_endpoint, "https://icp-api.io");
    assert_eq!(report.provenance.fetched_by, "ic-query");
    assert!(report.certification.certificate_verified);
    assert_eq!(
        format_utc_timestamp_secs(report.certification.certificate_time_nanos / 1_000_000_000),
        report.provenance.fetched_at
    );
}

#[tokio::test(flavor = "current_thread")]
async fn registry_version_report_rejects_custom_source_provenance_mismatch() {
    let request = NnsRegistryVersionRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
    };

    let error =
        build_nns_registry_version_report_with_source(&request, &MismatchedNnsRegistrySource)
            .await
            .expect_err("source provenance mismatch must fail");

    assert!(matches!(
        error,
        NnsRegistryHostError::SourceMismatch {
            field: "source_endpoint",
            expected,
            actual,
        } if expected == "https://icp-api.io" && actual == "https://wrong.example"
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn registry_version_report_rejects_unverified_custom_source_evidence() {
    let request =
        NnsRegistryVersionRequest::new(MAINNET_NETWORK, "https://icp-api.io", 1_780_531_200);

    let error =
        build_nns_registry_version_report_with_source(&request, &UnverifiedNnsRegistrySource)
            .await
            .expect_err("unverified custom evidence must fail");

    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("certificate_verified")
    ));
}

struct FixtureNnsRegistrySource;

impl NnsRegistrySource for FixtureNnsRegistrySource {
    fn fetch_registry_version<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<crate::nns::registry::NnsRegistryVersionData, NnsRegistryHostError>,
    > {
        Box::pin(async move {
            Ok(crate::nns::registry::NnsRegistryVersionData {
                provenance: crate::nns::registry::NnsRegistryInventoryProvenance {
                    network: MAINNET_NETWORK.to_string(),
                    registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
                    registry_version: 42,
                    fetched_at: request.fetched_at.clone(),
                    source_endpoint: request.endpoint.clone(),
                    fetched_by: request.fetched_by.clone(),
                },
                certification: certification(1_780_531_200),
            })
        })
    }
}

struct MismatchedNnsRegistrySource;

impl NnsRegistrySource for MismatchedNnsRegistrySource {
    fn fetch_registry_version<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<crate::nns::registry::NnsRegistryVersionData, NnsRegistryHostError>,
    > {
        Box::pin(async move {
            Ok(crate::nns::registry::NnsRegistryVersionData {
                provenance: crate::nns::registry::NnsRegistryInventoryProvenance {
                    network: request.network.clone(),
                    registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
                    registry_version: 42,
                    fetched_at: request.fetched_at.clone(),
                    source_endpoint: "https://wrong.example".to_string(),
                    fetched_by: request.fetched_by.clone(),
                },
                certification: certification(1_780_531_200),
            })
        })
    }
}

struct UnverifiedNnsRegistrySource;

impl NnsRegistrySource for UnverifiedNnsRegistrySource {
    fn fetch_registry_version<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<
        'a,
        Result<crate::nns::registry::NnsRegistryVersionData, NnsRegistryHostError>,
    > {
        Box::pin(async move {
            let mut evidence = certification(1_780_531_200);
            evidence.certificate_verified = false;
            Ok(crate::nns::registry::NnsRegistryVersionData {
                provenance: crate::nns::registry::NnsRegistryInventoryProvenance {
                    network: request.network.clone(),
                    registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
                    registry_version: 42,
                    fetched_at: request.fetched_at.clone(),
                    source_endpoint: request.endpoint.clone(),
                    fetched_by: request.fetched_by.clone(),
                },
                certification: evidence,
            })
        })
    }
}

fn certification(timestamp_seconds: u64) -> NnsRegistryCertification {
    NnsRegistryCertification {
        certificate_verified: true,
        certificate_time_nanos: timestamp_seconds * 1_000_000_000,

        root_key_digest: "ab".repeat(32),
        certificate_hex: "cd".repeat(8),

        hash_tree_hex: "ef".repeat(4),
    }
}
