use super::source::nns_certified_registry_delta_limits;
use super::{
    NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
    NnsCertifiedRegistryDeltaSource, NnsCertifiedRegistryDeltaSourceFuture,
    NnsCertifiedRegistryDeltaVersion, NnsCertifiedRegistryMutation,
    NnsCertifiedRegistryMutationKind, NnsCertifiedRegistryValueEncoding, NnsRegistryCertification,
    NnsRegistryHostError, NnsRegistrySource, NnsRegistryVersionData, NnsRegistryVersionReport,
    NnsRegistryVersionRequest, build_nns_registry_version_report_with_source,
    fetch_nns_certified_registry_delta_batch_with_source_async, nns_registry_version_report_text,
    validate_nns_certified_registry_delta_batch,
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
fn live_certified_delta_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsCertifiedRegistryDeltaBatchRequest::new(
        "local",
        "not a valid replica endpoint",
        41,
        1_780_531_200,
    );

    let error = crate::runtime::block_on_current_thread(
        LiveNnsSource.fetch_certified_registry_delta_batch(&request),
    )
    .expect("test runtime")
    .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsRegistryHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn certified_delta_public_builder_validates_custom_source_evidence() {
    let request = certified_delta_request();

    let report = crate::runtime::block_on_current_thread(
        fetch_nns_certified_registry_delta_batch_with_source_async(
            &request,
            &FixtureCertifiedDeltaSource,
        ),
    )
    .expect("test runtime")
    .expect("certified delta report");

    assert_eq!(report.requested_version, 41);
    assert_eq!(report.certified_latest_version, 43);
    assert_eq!(report.first_version, Some(42));
    assert_eq!(report.last_version, Some(42));
    assert!(report.more_available);
    assert_eq!(
        report.versions[0].mutations[0].mutation_kind,
        NnsCertifiedRegistryMutationKind::Upsert
    );
}

#[test]
fn certified_delta_pure_validator_rejects_sequence_and_derived_field_tampering() {
    let request = certified_delta_request();
    let mut report = certified_delta_report(&request);
    report.versions[0].version = 43;
    report.first_version = Some(43);
    report.last_version = Some(43);
    let error = validate_nns_certified_registry_delta_batch(&request, &report)
        .expect_err("wrong first version");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("version sequence expected 42")
    ));

    let mut report = certified_delta_report(&request);
    report.mutation_count = 2;
    let error = validate_nns_certified_registry_delta_batch(&request, &report)
        .expect_err("derived count mismatch");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("mutation_count mismatch")
    ));

    let mut report = certified_delta_report(&request);
    report.versions[0].mutations[0].mutation_type = 1;
    let error = validate_nns_certified_registry_delta_batch(&request, &report)
        .expect_err("raw and typed mutation mismatch");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("does not match kind")
    ));
}

#[test]
fn certified_delta_validator_accepts_bounded_chunk_evidence_and_recomputes_accounting() {
    let request = certified_delta_request();
    let mut report = certified_delta_report(&request);
    let hash = "ab".repeat(32);
    let mutation = &mut report.versions[0].mutations[0];
    mutation.value_encoding = NnsCertifiedRegistryValueEncoding::Chunked;
    mutation.chunk_sha256_hexes = vec![hash.clone(), hash];
    mutation.value_hex = Some("6263".to_string());
    report.inline_value_bytes = 0;
    report.chunk_value_bytes = 2;
    report.value_bytes = 2;
    report.chunk_reference_count = 2;
    report.chunk_query_call_count = 1;
    report.query_call_count = 2;
    report.chunk_response_bytes = 32;
    report.response_bytes = 96;

    validate_nns_certified_registry_delta_batch(&request, &report).expect("bounded chunk evidence");

    let mut invalid_hash = report.clone();
    invalid_hash.versions[0].mutations[0].chunk_sha256_hexes[0] = "AB".repeat(32);
    let error = validate_nns_certified_registry_delta_batch(&request, &invalid_hash)
        .expect_err("noncanonical chunk hash");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("chunk SHA-256")
    ));

    let mut hidden_query = report;
    hidden_query.chunk_query_call_count = 2;
    hidden_query.query_call_count = 3;
    let error = validate_nns_certified_registry_delta_batch(&request, &hidden_query)
        .expect_err("query count must equal unique chunk hashes");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("chunk_query_call_count mismatch")
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

struct FixtureCertifiedDeltaSource;

impl NnsCertifiedRegistryDeltaSource for FixtureCertifiedDeltaSource {
    fn fetch_certified_registry_delta_batch<'a>(
        &'a self,
        request: &'a NnsCertifiedRegistryDeltaBatchRequest,
    ) -> NnsCertifiedRegistryDeltaSourceFuture<'a> {
        Box::pin(async move { Ok(certified_delta_report(request)) })
    }
}

fn certified_delta_request() -> NnsCertifiedRegistryDeltaBatchRequest {
    NnsCertifiedRegistryDeltaBatchRequest::new(
        MAINNET_NETWORK,
        "https://icp-api.io",
        41,
        1_780_531_200,
    )
}

fn certified_delta_report(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
) -> NnsCertifiedRegistryDeltaBatchReport {
    NnsCertifiedRegistryDeltaBatchReport {
        schema_version: 2,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        requested_version: request.requested_version,
        certified_latest_version: 43,
        first_version: Some(42),
        last_version: Some(42),
        version_count: 1,
        mutation_count: 1,
        precondition_count: 0,
        inline_value_bytes: 1,
        chunk_value_bytes: 0,
        value_bytes: 1,
        chunk_reference_count: 0,
        more_available: true,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        query_call_count: 1,
        chunk_query_call_count: 0,
        certified_response_bytes: 64,
        chunk_response_bytes: 0,
        response_bytes: 64,
        limits: nns_certified_registry_delta_limits(),
        versions: vec![NnsCertifiedRegistryDeltaVersion {
            version: 42,
            timestamp_nanoseconds: 1_780_531_199_000_000_000,
            mutations: vec![NnsCertifiedRegistryMutation {
                mutation_type: 4,
                mutation_kind: NnsCertifiedRegistryMutationKind::Upsert,
                key_hex: "61".to_string(),
                value_encoding: NnsCertifiedRegistryValueEncoding::Inline,
                chunk_sha256_hexes: Vec::new(),
                value_hex: Some("62".to_string()),
            }],
            preconditions: Vec::new(),
        }],
        certification: certification(1_780_531_200),
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
