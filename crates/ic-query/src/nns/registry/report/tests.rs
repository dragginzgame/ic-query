use super::NnsCertifiedRegistryChunkEvidence;
use super::NnsCertifiedRegistryDeltaBatchReport;
use super::NnsCertifiedRegistryDeltaBatchRequest;
use super::NnsCertifiedRegistryDeltaSource;
use super::NnsCertifiedRegistryDeltaSourceFuture;
use super::NnsCertifiedRegistryDeltaVersion;
use super::NnsCertifiedRegistryMutation;
use super::NnsCertifiedRegistryMutationKind;
use super::NnsCertifiedRegistryValueEncoding;
use super::NnsRegistryCertification;
use super::NnsRegistryHostError;
use super::NnsRegistrySource;
use super::NnsRegistryVersionRequest;
use super::build_nns_registry_version_report_with_source;
use super::fetch_nns_certified_registry_delta_batch_with_source_async;
use super::source::nns_certified_registry_delta_limits;
use super::validate_nns_certified_registry_delta_batch;
use crate::nns::LiveNnsSource;
use crate::report::CollectionSourceRequest;
use crate::subnet_catalog::MAINNET_NETWORK;
use crate::subnet_catalog::MAINNET_REGISTRY_CANISTER_ID;
use ic_host_artifacts::artifact::Sha256Digest;
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
async fn live_certified_delta_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsCertifiedRegistryDeltaBatchRequest::new(
        "local",
        "not a valid replica endpoint",
        41,
        1_780_531_200,
    );

    let error = (LiveNnsSource::default().fetch_certified_registry_delta_batch(&request))
        .await
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsRegistryHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn certified_delta_public_builder_validates_custom_source_evidence() {
    let request = certified_delta_request();

    let report = fetch_nns_certified_registry_delta_batch_with_source_async(
        &request,
        &FixtureCertifiedDeltaSource,
    )
    .await
    .expect("certified delta report");

    assert_eq!(report.requested_version, 41);
    assert_eq!(report.certified_latest_version, 43);
    assert_eq!(report.summary().unwrap().first_version, Some(42));
    assert_eq!(report.summary().unwrap().last_version, Some(42));
    assert!(report.summary().unwrap().more_available);
    assert_eq!(
        report.versions[0].mutations[0].mutation_kind(),
        Some(NnsCertifiedRegistryMutationKind::Upsert)
    );
}

#[test]
fn certified_delta_validator_rejects_invalid_schema_sequence_and_operation() {
    let request = certified_delta_request();
    let mut report = certified_delta_report(&request);
    report.schema_version = u32::MAX;
    let error = validate_nns_certified_registry_delta_batch(&request, &report)
        .expect_err("unsupported report schema");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("schema_version mismatch")
    ));

    let mut report = certified_delta_report(&request);
    report.versions[0].version = 43;

    let error = validate_nns_certified_registry_delta_batch(&request, &report)
        .expect_err("wrong first version");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("version sequence expected 42")
    ));

    let mut report = certified_delta_report(&request);
    report.versions[0].mutations[0].mutation_type = 3;
    let error = validate_nns_certified_registry_delta_batch(&request, &report)
        .expect_err("unsupported raw mutation");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("unsupported mutation type")
    ));
}

#[test]
fn certified_delta_validator_accepts_bounded_chunk_evidence_and_recomputes_accounting() {
    let request = certified_delta_request();
    let mut report = certified_delta_report(&request);
    let content = b"bc";
    let hash = Sha256Digest::compute(content).to_string();
    let mutation = &mut report.versions[0].mutations[0];
    mutation.value_encoding = NnsCertifiedRegistryValueEncoding::Chunked;
    mutation.chunk_sha256_hexes = vec![hash.clone(), hash.clone()];
    mutation.value_hex = Some("62636263".to_string());

    report.chunk_evidence = vec![NnsCertifiedRegistryChunkEvidence {
        sha256_hex: hash,
        content_hex: crate::hex::hex_bytes(content),
    }];
    report.chunk_query_call_count = 1;

    report.chunk_response_bytes = 32;

    let summary = validate_nns_certified_registry_delta_batch(&request, &report)
        .expect("bounded chunk evidence");
    assert_eq!(summary.version_count, 1);
    assert_eq!(summary.mutation_count, 1);
    assert_eq!(summary.inline_value_bytes, 0);
    assert_eq!(summary.chunk_value_bytes, 4);
    assert_eq!(summary.value_bytes, 4);
    assert_eq!(summary.chunk_reference_count, 2);
    assert_eq!(summary.chunk_evidence_bytes, 2);
    assert_eq!(summary.query_call_count, 2);
    assert_eq!(summary.response_bytes, 96);

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

    let error = validate_nns_certified_registry_delta_batch(&request, &hidden_query)
        .expect_err("query count must equal unique chunk hashes");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("chunk_query_call_count mismatch")
    ));
}

#[test]
fn retained_registry_reports_refuse_retired_projection_fields() {
    let report = certified_delta_report(&certified_delta_request());
    for (pointer, field, value) in [
        ("", "mutation_count", serde_json::json!(1)),
        ("/certification", "certificate_bytes", serde_json::json!(8)),
        (
            "/versions/0/mutations/0",
            "mutation_kind",
            serde_json::json!("upsert"),
        ),
    ] {
        let mut encoded = serde_json::to_value(&report).unwrap();
        encoded.pointer_mut(pointer).unwrap()[field] = value;
        assert!(serde_json::from_value::<NnsCertifiedRegistryDeltaBatchReport>(encoded).is_err());
    }
}

#[test]
fn certified_delta_validator_rehashes_and_reconstructs_canonical_chunk_evidence() {
    let request = certified_delta_request();
    let mut report = certified_delta_report(&request);
    let content = b"chunk";
    let hash = Sha256Digest::compute(content).to_string();
    report.versions[0].mutations[0].value_encoding = NnsCertifiedRegistryValueEncoding::Chunked;
    report.versions[0].mutations[0].chunk_sha256_hexes = vec![hash.clone()];
    report.versions[0].mutations[0].value_hex = Some(crate::hex::hex_bytes(content));

    report.chunk_evidence = vec![NnsCertifiedRegistryChunkEvidence {
        sha256_hex: hash,
        content_hex: crate::hex::hex_bytes(content),
    }];
    report.chunk_query_call_count = 1;

    report.chunk_response_bytes = 32;

    let mut wrong_content = report.clone();
    wrong_content.chunk_evidence[0].content_hex = "00".to_string();
    let error = validate_nns_certified_registry_delta_batch(&request, &wrong_content)
        .expect_err("content hash mismatch");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("does not match SHA-256")
    ));

    let mut wrong_value = report.clone();
    wrong_value.versions[0].mutations[0].value_hex = Some("00".repeat(content.len()));
    let error = validate_nns_certified_registry_delta_batch(&request, &wrong_value)
        .expect_err("reconstructed value mismatch");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("ordered chunk evidence")
    ));

    let mut missing = report.clone();
    missing.chunk_evidence.clear();

    let error = validate_nns_certified_registry_delta_batch(&request, &missing)
        .expect_err("missing evidence");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("exactly the unique digests")
    ));

    let mut duplicate = report.clone();
    duplicate
        .chunk_evidence
        .push(duplicate.chunk_evidence[0].clone());

    let error = validate_nns_certified_registry_delta_batch(&request, &duplicate)
        .expect_err("duplicate evidence");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("unique and strictly ordered")
    ));

    let extra_content = b"unreferenced";
    let extra = NnsCertifiedRegistryChunkEvidence {
        sha256_hex: Sha256Digest::compute(extra_content).to_string(),
        content_hex: crate::hex::hex_bytes(extra_content),
    };
    let mut unreferenced = report.clone();

    unreferenced.chunk_evidence.push(extra);
    unreferenced
        .chunk_evidence
        .sort_by(|left, right| left.sha256_hex.cmp(&right.sha256_hex));
    let error = validate_nns_certified_registry_delta_batch(&request, &unreferenced)
        .expect_err("unreferenced evidence");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("exactly the unique digests")
    ));

    let mut noncanonical = unreferenced;
    noncanonical.chunk_evidence.reverse();
    let error = validate_nns_certified_registry_delta_batch(&request, &noncanonical)
        .expect_err("noncanonical evidence order");
    assert!(matches!(
        error,
        NnsRegistryHostError::InvalidSourceData { reason }
            if reason.contains("unique and strictly ordered")
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
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        requested_version: request.requested_version,
        certified_latest_version: 43,

        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),

        chunk_query_call_count: 0,
        certified_response_bytes: 64,
        chunk_response_bytes: 0,

        limits: nns_certified_registry_delta_limits(),
        versions: vec![NnsCertifiedRegistryDeltaVersion {
            version: 42,
            timestamp_nanoseconds: 1_780_531_199_000_000_000,
            mutations: vec![NnsCertifiedRegistryMutation {
                mutation_type: 4,

                key_hex: "61".to_string(),
                value_encoding: NnsCertifiedRegistryValueEncoding::Inline,
                chunk_sha256_hexes: Vec::new(),
                value_hex: Some("62".to_string()),
            }],
            preconditions: Vec::new(),
        }],
        chunk_evidence: Vec::new(),
        certification: certification(1_780_531_200),
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
