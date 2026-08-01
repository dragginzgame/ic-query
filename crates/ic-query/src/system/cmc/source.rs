//! Module: system::cmc::source
//!
//! Responsibility: query and authenticate the CMC certified ICP/XDR rate.
//! Does not own: report projection, cache policy, CLI parsing, or process output.
//! Boundary: one bounded native canister query yields one authenticated rate value.

use super::{
    CmcCertification, CmcCertifiedRate, CmcHostError, CmcIcpXdrConversionRate,
    MAINNET_CMC_CANISTER_ID, enforce_mainnet_network, wire::CmcCertifiedRateResponse,
};
use crate::{
    agent::build_ic_agent,
    certification::{CertifiedDataError, authenticate_canister_hash_tree},
    hex::hex_bytes,
    runtime::block_on_current_thread,
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::Principal;
use ic_agent::hash_tree::{HashTree, LookupResult};

const GET_ICP_XDR_CONVERSION_RATE_METHOD: &str = "get_icp_xdr_conversion_rate";
const ICP_XDR_CONVERSION_RATE_LABEL: &[u8] = b"ICP_XDR_CONVERSION_RATE";

///
/// CmcSourceRequest
///
/// Network and collection provenance for one direct CMC query.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CmcSourceRequest {
    /// Network to query.
    pub network: String,
    /// Replica endpoint used for the query.
    pub endpoint: String,
    /// UTC collection timestamp recorded in the report.
    pub fetched_at: String,
    /// Collector identity recorded in the report.
    pub fetched_by: String,
}

impl CmcSourceRequest {
    /// Create source settings for one CMC query.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }

    /// Create source settings from a Unix collection timestamp.
    #[must_use]
    pub fn from_unix_secs(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at_unix_secs: u64,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self::new(
            network,
            endpoint,
            format_utc_timestamp_secs(fetched_at_unix_secs),
            fetched_by,
        )
    }
}

///
/// CmcSource
///
/// Source capability for the certified CMC ICP/XDR conversion rate.
///

pub trait CmcSource {
    /// Fetch and authenticate the native CMC ICP/XDR conversion rate.
    fn fetch_certified_icp_xdr_rate(
        &self,
        request: &CmcSourceRequest,
    ) -> Result<CmcCertifiedRate, CmcHostError>;
}

///
/// LiveCmcSource
///
/// Built-in live adapter for the mainnet Cycle Minting Canister.
///

#[derive(Clone, Copy, Debug, Default)]
pub struct LiveCmcSource;

impl CmcSource for LiveCmcSource {
    fn fetch_certified_icp_xdr_rate(
        &self,
        request: &CmcSourceRequest,
    ) -> Result<CmcCertifiedRate, CmcHostError> {
        enforce_mainnet_network(&request.network)?;
        block_on_current_thread(fetch_live_certified_rate(request))?
    }
}

async fn fetch_live_certified_rate(
    request: &CmcSourceRequest,
) -> Result<CmcCertifiedRate, CmcHostError> {
    let agent = build_ic_agent(&request.endpoint, |reason| CmcHostError::AgentBuild {
        endpoint: request.endpoint.clone(),
        reason,
    })?;
    let canister = Principal::from_text(MAINNET_CMC_CANISTER_ID).map_err(|error| {
        CmcHostError::CanisterId {
            reason: error.to_string(),
        }
    })?;
    let arg = candid::encode_args(()).map_err(|error| CmcHostError::CandidEncode {
        message: "()",
        reason: error.to_string(),
    })?;
    let bytes = agent
        .query(&canister, GET_ICP_XDR_CONVERSION_RATE_METHOD)
        .with_arg(arg)
        .call()
        .await
        .map_err(|error| CmcHostError::AgentCall {
            method: GET_ICP_XDR_CONVERSION_RATE_METHOD,
            reason: error.to_string(),
        })?;
    let response: CmcCertifiedRateResponse =
        candid::decode_one(&bytes).map_err(|error| CmcHostError::CandidDecode {
            message: "IcpXdrConversionRateResponse",
            reason: error.to_string(),
        })?;
    verified_certified_rate(&agent, &canister, response)
}

fn verified_certified_rate(
    agent: &ic_agent::Agent,
    canister: &Principal,
    response: CmcCertifiedRateResponse,
) -> Result<CmcCertifiedRate, CmcHostError> {
    let hash_tree = authenticate_canister_hash_tree(
        agent,
        canister,
        &response.certificate,
        &response.hash_tree,
        "canister",
    )
    .map_err(map_certified_data_error)?;
    verify_rate_leaf(&hash_tree, &response.data)?;

    Ok(CmcCertifiedRate {
        rate: response.data,
        certification: CmcCertification {
            certificate_verified: true,
            certificate_hex: hex_bytes(&response.certificate),
            certificate_bytes: response.certificate.len(),
            hash_tree_hex: hex_bytes(&response.hash_tree),
            hash_tree_bytes: response.hash_tree.len(),
        },
    })
}

fn verify_rate_leaf(
    hash_tree: &HashTree<Vec<u8>>,
    rate: &CmcIcpXdrConversionRate,
) -> Result<(), CmcHostError> {
    let expected = candid::encode_one(rate).map_err(|error| CmcHostError::CandidEncode {
        message: "IcpXdrConversionRate",
        reason: error.to_string(),
    })?;
    let actual = match hash_tree.lookup_path([ICP_XDR_CONVERSION_RATE_LABEL]) {
        LookupResult::Found(value) => value,
        LookupResult::Absent => return Err(missing_rate_leaf("absent")),
        LookupResult::Unknown => {
            return Err(missing_rate_leaf("not proven by the partial tree"));
        }
        LookupResult::Error => return Err(missing_rate_leaf("not a leaf")),
    };
    if actual != expected {
        return Err(CmcHostError::InvalidCertifiedRate {
            reason: "ICP_XDR_CONVERSION_RATE leaf does not match the returned Candid rate"
                .to_string(),
        });
    }
    Ok(())
}

fn missing_rate_leaf(state: &str) -> CmcHostError {
    CmcHostError::InvalidCertifiedRate {
        reason: format!("required ICP_XDR_CONVERSION_RATE leaf is {state}"),
    }
}

fn map_certified_data_error(error: CertifiedDataError) -> CmcHostError {
    match error {
        CertifiedDataError::Authentication { reason } => {
            CmcHostError::CertificateAuthentication { reason }
        }
        CertifiedDataError::Invalid { reason } => CmcHostError::InvalidCertifiedRate { reason },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certification::verify_canister_hash_tree;
    use ic_agent::{
        Certificate,
        hash_tree::{label, leaf},
    };

    #[test]
    fn official_candid_shape_round_trips() {
        let response = response(Vec::new(), Vec::new());
        let bytes = candid::encode_one(&response).expect("encode official CMC response");
        let decoded: CmcCertifiedRateResponse =
            candid::decode_one(&bytes).expect("decode official CMC response");

        assert_eq!(decoded, response);
    }

    #[test]
    fn accepts_a_rate_leaf_committed_by_certified_data() {
        let canister = cmc_canister();
        let rate = rate();
        let encoded_rate = candid::encode_one(&rate).expect("encode CMC rate");
        let hash_tree = label(ICP_XDR_CONVERSION_RATE_LABEL.to_vec(), leaf(encoded_rate));
        let certificate = certificate_for_tree(&canister, hash_tree.digest().to_vec());
        let encoded_tree = serde_cbor::to_vec(&hash_tree).expect("encode hash tree");

        let verified_tree =
            verify_canister_hash_tree(&certificate, &canister, &encoded_tree, "canister")
                .expect("matching certified-data witness");
        verify_rate_leaf(&verified_tree, &rate).expect("matching certified CMC rate leaf");
    }

    #[test]
    fn rejects_a_leaf_for_a_different_rate() {
        let rate = rate();
        let different_rate = CmcIcpXdrConversionRate {
            timestamp_seconds: rate.timestamp_seconds,
            xdr_permyriad_per_icp: rate.xdr_permyriad_per_icp + 1,
        };
        let encoded_rate = candid::encode_one(&different_rate).expect("encode CMC rate");
        let hash_tree = label(ICP_XDR_CONVERSION_RATE_LABEL.to_vec(), leaf(encoded_rate));

        let error = verify_rate_leaf(&hash_tree, &rate)
            .expect_err("a different Candid rate must not satisfy the witness");

        assert!(matches!(
            error,
            CmcHostError::InvalidCertifiedRate { reason }
                if reason.contains("does not match")
        ));
    }

    #[test]
    fn rejects_a_missing_rate_leaf() {
        let hash_tree = label(b"OTHER".to_vec(), leaf(vec![0]));
        let error = verify_rate_leaf(&hash_tree, &rate())
            .expect_err("the native CMC witness label is required");

        assert!(matches!(
            error,
            CmcHostError::InvalidCertifiedRate { reason }
                if reason.contains("leaf is absent")
        ));
    }

    #[test]
    fn authenticates_the_certificate_before_accepting_the_rate() {
        let canister = cmc_canister();
        let rate = rate();
        let encoded_rate = candid::encode_one(&rate).expect("encode CMC rate");
        let hash_tree = label(ICP_XDR_CONVERSION_RATE_LABEL.to_vec(), leaf(encoded_rate));
        let certificate = certificate_for_tree(&canister, hash_tree.digest().to_vec());
        let response = response(
            serde_cbor::to_vec(&hash_tree).expect("encode hash tree"),
            serde_cbor::to_vec(&certificate).expect("encode certificate"),
        );
        let agent = ic_agent::Agent::builder()
            .with_url(super::super::DEFAULT_CMC_SOURCE_ENDPOINT)
            .build()
            .expect("build agent");

        let error = verified_certified_rate(&agent, &canister, response)
            .expect_err("an unsigned certificate must fail authentication");

        assert!(matches!(
            error,
            CmcHostError::CertificateAuthentication { .. }
        ));
    }

    fn rate() -> CmcIcpXdrConversionRate {
        CmcIcpXdrConversionRate {
            timestamp_seconds: 1_722_510_000,
            xdr_permyriad_per_icp: 49_164,
        }
    }

    fn response(hash_tree: Vec<u8>, certificate: Vec<u8>) -> CmcCertifiedRateResponse {
        CmcCertifiedRateResponse {
            data: rate(),
            hash_tree,
            certificate,
        }
    }

    fn cmc_canister() -> Principal {
        Principal::from_text(MAINNET_CMC_CANISTER_ID).expect("valid CMC canister")
    }

    fn certificate_for_tree(canister: &Principal, certified_data: Vec<u8>) -> Certificate {
        Certificate {
            tree: label(
                b"canister".to_vec(),
                label(
                    canister.as_slice().to_vec(),
                    label(b"certified_data".to_vec(), leaf(certified_data)),
                ),
            ),
            signature: Vec::new(),
            delegation: None,
        }
    }
}
