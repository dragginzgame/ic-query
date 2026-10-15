//! Module: system::cmc::build
//!
//! Responsibility: assemble CMC reports and exact cycles derivations.
//! Does not own: live transport, certificate validation, CLI parsing, or rendering.
//! Boundary: validates mainnet identity before invoking any source capability.

use super::{
    CMC_REPORT_SCHEMA_VERSION, CYCLES_PER_XDR, CmcCertifiedRate, CmcCyclesReport, CmcHostError,
    CmcReportContext, CmcSource, CmcSourceRequest, CmcXdrReport, ICP_XDR_PERMYRIAD_DENOMINATOR,
    LiveCmcSource, MAINNET_CMC_CANISTER_ID, enforce_mainnet_network,
};
use crate::hex::{HexEvidenceError, validate_nonempty_hex_evidence};

const CYCLES_PER_ICP_FORMULA: &str = "xdr_permyriad_per_icp * cycles_per_xdr / 10000";
const CYCLES_PER_XDR_SOURCE: &str = "ic_protocol_constant";

/// Build one live certified CMC ICP/XDR conversion-rate report.
pub async fn build_cmc_xdr_report(
    request: &CmcSourceRequest,
) -> Result<CmcXdrReport, CmcHostError> {
    build_cmc_xdr_report_with_source(request, &LiveCmcSource).await
}

/// Build one certified CMC ICP/XDR report from a custom source.
pub async fn build_cmc_xdr_report_with_source(
    request: &CmcSourceRequest,
    source: &dyn CmcSource,
) -> Result<CmcXdrReport, CmcHostError> {
    enforce_mainnet_network(&request.network)?;
    let certified = source.fetch_certified_icp_xdr_rate(request).await?;
    validate_certified_rate(&certified)?;
    Ok(CmcXdrReport {
        context: report_context(request),
        rate: certified.rate,
        certification: certified.certification,
    })
}

/// Build one live cycles conversion report from the certified CMC ICP/XDR rate.
pub async fn build_cmc_cycles_report(
    request: &CmcSourceRequest,
) -> Result<CmcCyclesReport, CmcHostError> {
    build_cmc_cycles_report_with_source(request, &LiveCmcSource).await
}

/// Build one cycles conversion report from a custom certified CMC source.
pub async fn build_cmc_cycles_report_with_source(
    request: &CmcSourceRequest,
    source: &dyn CmcSource,
) -> Result<CmcCyclesReport, CmcHostError> {
    enforce_mainnet_network(&request.network)?;
    let certified = source.fetch_certified_icp_xdr_rate(request).await?;
    validate_certified_rate(&certified)?;
    let cycles_per_icp = u128::from(certified.rate.xdr_permyriad_per_icp) * CYCLES_PER_XDR
        / ICP_XDR_PERMYRIAD_DENOMINATOR;
    Ok(CmcCyclesReport {
        context: report_context(request),
        rate: certified.rate,
        cycles_per_xdr: CYCLES_PER_XDR,
        cycles_per_xdr_source: CYCLES_PER_XDR_SOURCE.to_string(),
        cycles_per_icp,
        cycles_per_icp_formula: CYCLES_PER_ICP_FORMULA.to_string(),
        certification: certified.certification,
    })
}

fn report_context(request: &CmcSourceRequest) -> CmcReportContext {
    CmcReportContext {
        schema_version: CMC_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cmc_canister_id: MAINNET_CMC_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        source_endpoint: request.endpoint.clone(),
        fetched_by: request.fetched_by.clone(),
    }
}

fn validate_certified_rate(certified: &CmcCertifiedRate) -> Result<(), CmcHostError> {
    if !certified.certification.certificate_verified {
        return Err(invalid_source_data(
            "certificate_verified must be true for CmcSource results",
        ));
    }
    validate_evidence_hex(
        "certificate_hex",
        &certified.certification.certificate_hex,
        certified.certification.certificate_bytes,
    )?;
    validate_evidence_hex(
        "hash_tree_hex",
        &certified.certification.hash_tree_hex,
        certified.certification.hash_tree_bytes,
    )
}

fn validate_evidence_hex(field: &str, value: &str, byte_count: usize) -> Result<(), CmcHostError> {
    validate_nonempty_hex_evidence(value, byte_count).map_err(|error| {
        invalid_source_data(match error {
            HexEvidenceError::Empty => format!("{field} evidence must not be empty"),
            HexEvidenceError::ByteCountOverflow => format!("{field} byte count is too large"),
            HexEvidenceError::LengthMismatch => format!(
                "{field} length {} does not match {byte_count} bytes",
                value.len()
            ),
            HexEvidenceError::NonLowercaseHex => {
                format!("{field} must be canonical lowercase hexadecimal")
            }
        })
    })
}

fn invalid_source_data(reason: impl Into<String>) -> CmcHostError {
    CmcHostError::InvalidSourceData {
        reason: reason.into(),
    }
}
