use ic_query::system::cmc::{
    CYCLES_PER_XDR, CmcCertification, CmcCyclesReport, CmcIcpXdrConversionRate, CmcReportContext,
    CmcXdrReport, DEFAULT_CMC_SOURCE_ENDPOINT, MAINNET_CMC_CANISTER_ID, cmc_cycles_report_text,
    cmc_xdr_report_text,
};
#[cfg(feature = "host")]
use ic_query::system::cmc::{
    CmcCertifiedRate, CmcHostError, CmcSource, CmcSourceRequest,
    build_cmc_cycles_report_with_source, build_cmc_xdr_report_with_source,
};

#[test]
fn public_cmc_reports_are_constructible_serializable_and_renderable() {
    let xdr = CmcXdrReport {
        context: context(),
        rate: rate(),
        certification: certification(),
    };
    let cycles = CmcCyclesReport {
        context: context(),
        rate: rate(),
        cycles_per_xdr: CYCLES_PER_XDR,
        cycles_per_xdr_source: "ic_protocol_constant".to_string(),
        cycles_per_icp: 4_916_400_000_000,
        cycles_per_icp_formula: "xdr_permyriad_per_icp * cycles_per_xdr / 10000".to_string(),
        certification: certification(),
    };

    assert!(cmc_xdr_report_text(&xdr).contains("xdr_per_icp: 4.9164"));
    assert!(cmc_cycles_report_text(&cycles).contains("cycles_per_icp: 4916400000000"));

    let xdr_json = serde_json::to_value(xdr).expect("serialize public CMC XDR report");
    let cycles_json = serde_json::to_value(cycles).expect("serialize public CMC cycles report");
    assert_eq!(xdr_json["cmc_canister_id"], MAINNET_CMC_CANISTER_ID);
    assert_eq!(xdr_json["rate"]["xdr_permyriad_per_icp"], 49_164);
    assert_eq!(cycles_json["cycles_per_xdr"], 1_000_000_000_000_u64);
    assert_eq!(cycles_json["certification"]["certificate_verified"], true);
}

#[cfg(feature = "host")]
#[test]
fn public_host_api_accepts_a_custom_cmc_source() {
    let request = CmcSourceRequest::from_unix_secs(
        "ic",
        DEFAULT_CMC_SOURCE_ENDPOINT,
        1_700_000_000,
        "fixture",
    );

    let xdr = build_cmc_xdr_report_with_source(&request, &FixtureCmcSource)
        .expect("custom CMC XDR source");
    let cycles = build_cmc_cycles_report_with_source(&request, &FixtureCmcSource)
        .expect("custom CMC cycles source");

    assert_eq!(xdr.context.fetched_at, "2023-11-14T22:13:20Z");
    assert_eq!(cycles.cycles_per_icp, 4_916_400_000_000);
}

#[cfg(feature = "host")]
struct FixtureCmcSource;

#[cfg(feature = "host")]
impl CmcSource for FixtureCmcSource {
    fn fetch_certified_icp_xdr_rate(
        &self,
        _request: &CmcSourceRequest,
    ) -> Result<CmcCertifiedRate, CmcHostError> {
        Ok(CmcCertifiedRate {
            rate: rate(),
            certification: certification(),
        })
    }
}

fn context() -> CmcReportContext {
    CmcReportContext {
        schema_version: 1,
        network: "ic".to_string(),
        cmc_canister_id: MAINNET_CMC_CANISTER_ID.to_string(),
        fetched_at: "2026-08-01T12:00:00Z".to_string(),
        source_endpoint: DEFAULT_CMC_SOURCE_ENDPOINT.to_string(),
        fetched_by: "fixture".to_string(),
    }
}

const fn rate() -> CmcIcpXdrConversionRate {
    CmcIcpXdrConversionRate {
        timestamp_seconds: 1_722_510_000,
        xdr_permyriad_per_icp: 49_164,
    }
}

fn certification() -> CmcCertification {
    CmcCertification {
        certificate_verified: true,
        certificate_hex: "aabb".to_string(),
        certificate_bytes: 2,
        hash_tree_hex: "cc".to_string(),
        hash_tree_bytes: 1,
    }
}
