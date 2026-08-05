use super::{
    NnsRegistryCertification, NnsRegistryHostError, NnsRegistrySource, NnsRegistryVersionData,
    NnsRegistryVersionReport, NnsRegistryVersionRequest,
    build_nns_registry_version_report_with_source, nns_registry_version_report_text,
};
use crate::nns::{LiveNnsSource, NnsSourceRequest};
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID};

#[test]
fn live_registry_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = LiveNnsSource
        .fetch_registry_version(&request)
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsRegistryHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn registry_version_report_uses_live_source_shape() {
    let request = NnsRegistryVersionRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
    };

    let report = build_nns_registry_version_report_with_source(&request, &FixtureNnsRegistrySource)
        .expect("registry version report");

    assert_eq!(report.schema_version, 2);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(report.registry_version, 42);
    assert_eq!(report.fetched_at, "2026-06-04T00:00:00Z");
    assert_eq!(report.source_endpoint, "https://icp-api.io");
    assert_eq!(report.fetched_by, "ic-query");
    assert!(report.certification.certificate_verified);
    assert_eq!(report.certification.certificate_time, report.fetched_at);
}

#[test]
fn registry_version_report_rejects_custom_source_provenance_mismatch() {
    let request = NnsRegistryVersionRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
    };

    let error =
        build_nns_registry_version_report_with_source(&request, &MismatchedNnsRegistrySource)
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

#[test]
fn registry_version_report_rejects_unverified_custom_source_evidence() {
    let request =
        NnsRegistryVersionRequest::new(MAINNET_NETWORK, "https://icp-api.io", 1_780_531_200);

    let error =
        build_nns_registry_version_report_with_source(&request, &UnverifiedNnsRegistrySource)
            .expect_err("unverified custom evidence must fail");

    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("certificate_verified")
    ));
}

#[test]
fn registry_version_text_is_key_value_output() {
    let report = NnsRegistryVersionReport {
        schema_version: 2,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 42,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        certification: certification(1_780_531_200),
    };

    let text = nns_registry_version_report_text(&report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("registry_canister_id: rwlgt-iiaaa-aaaaa-aaaaa-cai"));
    assert!(text.contains("registry_version: 42"));
    assert!(text.contains("fetched_at: 2026-06-04T00:00:00Z"));
    assert!(text.contains("assurance: certified"));
    assert!(text.contains("certificate_verified: true"));
}

struct FixtureNnsRegistrySource;

impl NnsRegistrySource for FixtureNnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
        Ok(NnsRegistryVersionData {
            network: MAINNET_NETWORK.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            certification: certification(1_780_531_200),
        })
    }
}

struct MismatchedNnsRegistrySource;

impl NnsRegistrySource for MismatchedNnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
        Ok(NnsRegistryVersionData {
            network: request.network.clone(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: "https://wrong.example".to_string(),
            certification: certification(1_780_531_200),
        })
    }
}

struct UnverifiedNnsRegistrySource;

impl NnsRegistrySource for UnverifiedNnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
        let mut evidence = certification(1_780_531_200);
        evidence.certificate_verified = false;
        Ok(NnsRegistryVersionData {
            network: request.network.clone(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            certification: evidence,
        })
    }
}

fn certification(timestamp_seconds: u64) -> NnsRegistryCertification {
    NnsRegistryCertification {
        certificate_verified: true,
        certificate_time_nanos: timestamp_seconds * 1_000_000_000,
        certificate_time: crate::subnet_catalog::format_utc_timestamp_secs(timestamp_seconds),
        root_key_digest: "ab".repeat(32),
        certificate_hex: "cd".repeat(8),
        certificate_bytes: 8,
        hash_tree_hex: "ef".repeat(4),
        hash_tree_bytes: 4,
    }
}
