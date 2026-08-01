//! Module: system::cmc::wire
//!
//! Responsibility: mirror the public CMC certified conversion-rate Candid response.
//! Does not own: public report models, certificate validation, or transport.
//! Boundary: changes here follow the official CMC Candid interface exactly.

use super::CmcIcpXdrConversionRate;
use candid::{CandidType, Deserialize};

///
/// CmcCertifiedRateResponse
///
/// Wire response from `get_icp_xdr_conversion_rate`.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub(super) struct CmcCertifiedRateResponse {
    /// Latest native ICP/XDR rate.
    pub(super) data: CmcIcpXdrConversionRate,
    /// CBOR-serialized partial hash tree proving `data`.
    pub(super) hash_tree: Vec<u8>,
    /// CBOR-serialized IC system certificate.
    pub(super) certificate: Vec<u8>,
}
