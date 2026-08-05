//! Module: system::cmc::model
//!
//! Responsibility: define stable CMC report and certified-rate models.
//! Does not own: live transport, certificate verification, CLI parsing, or text output.
//! Boundary: preserves the native CMC permyriad rate and explicit evidence provenance.

#[cfg(feature = "cmc-host")]
use candid::CandidType;
use serde::{Deserialize, Serialize};

///
/// CmcReportContext
///
/// Provenance shared by direct Cycle Minting Canister reports.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CmcReportContext {
    /// Report schema version.
    pub schema_version: u32,
    /// Queried network identity.
    pub network: String,
    /// Cycle Minting Canister principal.
    pub cmc_canister_id: String,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Replica endpoint used for the query.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
}

///
/// CmcIcpXdrConversionRate
///
/// Native CMC ICP/XDR conversion-rate value committed by certified data.
///

#[cfg_attr(feature = "cmc-host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CmcIcpXdrConversionRate {
    /// Unix timestamp for the market data represented by this rate.
    pub timestamp_seconds: u64,
    /// Number of ten-thousandths of XDR corresponding to one ICP.
    pub xdr_permyriad_per_icp: u64,
}

///
/// CmcCertification
///
/// Authenticated application-level certificate and hash-tree evidence for a CMC rate.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CmcCertification {
    /// Whether the built-in source authenticated the certificate and rate witness.
    pub certificate_verified: bool,
    /// CBOR system certificate encoded as lowercase hexadecimal.
    pub certificate_hex: String,
    /// Raw certificate length in bytes.
    pub certificate_bytes: usize,
    /// CBOR witness hash tree encoded as lowercase hexadecimal.
    pub hash_tree_hex: String,
    /// Raw witness hash-tree length in bytes.
    pub hash_tree_bytes: usize,
}

///
/// CmcXdrReport
///
/// Certified point-in-time CMC ICP/XDR conversion-rate report.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CmcXdrReport {
    /// Query and authority provenance.
    #[serde(flatten)]
    pub context: CmcReportContext,
    /// Raw CMC conversion-rate value.
    pub rate: CmcIcpXdrConversionRate,
    /// Authenticated certificate and witness evidence.
    pub certification: CmcCertification,
}

///
/// CmcCyclesReport
///
/// Cycles conversion derived from a certified CMC rate and the IC protocol constant.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CmcCyclesReport {
    /// Query and authority provenance.
    #[serde(flatten)]
    pub context: CmcReportContext,
    /// Raw CMC conversion-rate value used by the derivation.
    pub rate: CmcIcpXdrConversionRate,
    /// IC protocol constant for the number of cycles corresponding to one XDR.
    pub cycles_per_xdr: u128,
    /// Authority label for `cycles_per_xdr`, distinct from the certified CMC rate.
    pub cycles_per_xdr_source: String,
    /// Number of cycles corresponding to one ICP at the certified rate.
    pub cycles_per_icp: u128,
    /// Exact formula used to derive `cycles_per_icp`.
    pub cycles_per_icp_formula: String,
    /// Authenticated certificate and witness evidence for `rate`.
    pub certification: CmcCertification,
}

///
/// CmcCertifiedRate
///
/// Source result accepted after the CMC certificate and rate witness are validated.
///

#[cfg(feature = "cmc-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CmcCertifiedRate {
    /// Native certified CMC rate.
    pub rate: CmcIcpXdrConversionRate,
    /// Authenticated certificate and hash-tree evidence.
    pub certification: CmcCertification,
}
