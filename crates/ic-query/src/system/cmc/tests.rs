use super::*;
use crate::subnet_catalog::MAINNET_NETWORK;
use std::cell::Cell;

const FETCHED_AT: &str = "2026-08-01T12:00:00Z";

struct FixtureCmcSource {
    calls: Cell<usize>,
}

impl FixtureCmcSource {
    const fn new() -> Self {
        Self {
            calls: Cell::new(0),
        }
    }
}

impl CmcSource for FixtureCmcSource {
    fn fetch_certified_icp_xdr_rate(
        &self,
        _request: &CmcSourceRequest,
    ) -> Result<CmcCertifiedRate, CmcHostError> {
        self.calls.set(self.calls.get() + 1);
        Ok(certified_rate())
    }
}

#[test]
fn xdr_report_preserves_raw_rate_and_certification() {
    let source = FixtureCmcSource::new();
    let report = build_cmc_xdr_report_with_source(&request(MAINNET_NETWORK), &source)
        .expect("fixture CMC report");

    assert_eq!(source.calls.get(), 1);
    assert_eq!(report.context.schema_version, 1);
    assert_eq!(report.context.network, MAINNET_NETWORK);
    assert_eq!(report.context.cmc_canister_id, MAINNET_CMC_CANISTER_ID);
    assert_eq!(report.rate.timestamp_seconds, 1_722_510_000);
    assert_eq!(report.rate.xdr_permyriad_per_icp, 49_164);
    assert!(report.certification.certificate_verified);

    let text = cmc_xdr_report_text(&report);
    assert!(text.contains("xdr_permyriad_per_icp: 49164"));
    assert!(text.contains("xdr_per_icp: 4.9164"));
    assert!(text.contains("certificate_verified: true"));

    let json = serde_json::to_value(&report).expect("serialize CMC XDR report");
    assert_eq!(json["xdr_permyriad_per_icp"], serde_json::Value::Null);
    assert_eq!(json["rate"]["xdr_permyriad_per_icp"], 49_164);
    assert_eq!(json["certification"]["certificate_hex"], "aabb");
}

#[test]
fn cycles_report_uses_the_exact_protocol_conversion() {
    let source = FixtureCmcSource::new();
    let report = build_cmc_cycles_report_with_source(&request(MAINNET_NETWORK), &source)
        .expect("fixture CMC cycles report");

    assert_eq!(source.calls.get(), 1);
    assert_eq!(report.cycles_per_xdr, 1_000_000_000_000);
    assert_eq!(report.cycles_per_xdr_source, "ic_protocol_constant");
    assert_eq!(report.cycles_per_icp, 4_916_400_000_000);
    assert_eq!(
        report.cycles_per_icp_formula,
        "xdr_permyriad_per_icp * cycles_per_xdr / 10000"
    );

    let text = cmc_cycles_report_text(&report);
    assert!(text.contains("cycles_per_xdr: 1 T"));
    assert!(text.contains("cycles_per_xdr_source: ic_protocol_constant"));
    assert!(text.contains("cycles_per_icp: 4.92 T"));
}

#[test]
fn builders_reject_non_mainnet_before_invoking_a_source() {
    for build in [
        |request: &CmcSourceRequest, source: &dyn CmcSource| {
            build_cmc_xdr_report_with_source(request, source).map(|_| ())
        },
        |request: &CmcSourceRequest, source: &dyn CmcSource| {
            build_cmc_cycles_report_with_source(request, source).map(|_| ())
        },
    ] {
        let source = FixtureCmcSource::new();
        let error = build(&request("local"), &source)
            .expect_err("non-mainnet CMC reports must be rejected");

        assert!(matches!(
            error,
            CmcHostError::UnsupportedNetwork { network } if network == "local"
        ));
        assert_eq!(source.calls.get(), 0);
    }
}

#[test]
fn public_live_source_rejects_non_mainnet_before_agent_construction() {
    let request = CmcSourceRequest::new("local", "://invalid endpoint", FETCHED_AT, "test");
    let error = LiveCmcSource
        .fetch_certified_icp_xdr_rate(&request)
        .expect_err("network must be validated before the endpoint");

    assert!(matches!(
        error,
        CmcHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn builders_reject_invalid_custom_certification_evidence() {
    struct InvalidSource;

    impl CmcSource for InvalidSource {
        fn fetch_certified_icp_xdr_rate(
            &self,
            _request: &CmcSourceRequest,
        ) -> Result<CmcCertifiedRate, CmcHostError> {
            let mut certified = certified_rate();
            certified.certification.certificate_verified = false;
            Ok(certified)
        }
    }

    let error = build_cmc_xdr_report_with_source(&request(MAINNET_NETWORK), &InvalidSource)
        .expect_err("custom sources must return accepted certified evidence");

    assert!(matches!(
        error,
        CmcHostError::InvalidSourceData { reason }
            if reason.contains("certificate_verified")
    ));
}

#[test]
fn builders_reject_inconsistent_custom_evidence_lengths() {
    struct InvalidSource;

    impl CmcSource for InvalidSource {
        fn fetch_certified_icp_xdr_rate(
            &self,
            _request: &CmcSourceRequest,
        ) -> Result<CmcCertifiedRate, CmcHostError> {
            let mut certified = certified_rate();
            certified.certification.hash_tree_bytes = 2;
            Ok(certified)
        }
    }

    let error = build_cmc_cycles_report_with_source(&request(MAINNET_NETWORK), &InvalidSource)
        .expect_err("custom source byte counts must match evidence hex");

    assert!(matches!(
        error,
        CmcHostError::InvalidSourceData { reason }
            if reason.contains("hash_tree_hex length")
    ));
}

#[test]
fn source_request_formats_unix_collection_time() {
    let request =
        CmcSourceRequest::from_unix_secs(MAINNET_NETWORK, DEFAULT_CMC_SOURCE_ENDPOINT, 0, "test");

    assert_eq!(request.fetched_at, "1970-01-01T00:00:00Z");
}

fn request(network: &str) -> CmcSourceRequest {
    CmcSourceRequest::new(network, DEFAULT_CMC_SOURCE_ENDPOINT, FETCHED_AT, "test")
}

fn certified_rate() -> CmcCertifiedRate {
    CmcCertifiedRate {
        rate: CmcIcpXdrConversionRate {
            timestamp_seconds: 1_722_510_000,
            xdr_permyriad_per_icp: 49_164,
        },
        certification: CmcCertification {
            certificate_verified: true,
            certificate_hex: "aabb".to_string(),
            certificate_bytes: 2,
            hash_tree_hex: "cc".to_string(),
            hash_tree_bytes: 1,
        },
    }
}
