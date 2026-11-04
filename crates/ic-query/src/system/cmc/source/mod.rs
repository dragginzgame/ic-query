//! Module: system::cmc::source
//!
//! Responsibility: query and authenticate the CMC certified ICP/XDR rate.
//! Does not own: report projection, cache policy, CLI parsing, or process output.
//! Boundary: one bounded native canister query yields one authenticated rate value.

use super::{
    CmcCertification, CmcCertifiedRate, CmcHostError, CmcIcpXdrConversionRate,
    MAINNET_CMC_CANISTER_ID, enforce_mainnet_network,
};
use crate::report::CollectionSourceRequest;
use crate::{
    agent::build_ic_agent,
    candid_call::{encode_request, query_encoded},
    certification::{CertifiedDataError, authenticate_canister_hash_tree},
    hex::hex_bytes,
};
use ic_platform::candid::contract::Principal;
use ic_platform::certification::{HashTree, LookupResult};
use ic_platform::cmc_types::IcpXdrConversionRateResponse;

const GET_ICP_XDR_CONVERSION_RATE_METHOD: &str = "get_icp_xdr_conversion_rate";
const ICP_XDR_CONVERSION_RATE_LABEL: &[u8] = b"ICP_XDR_CONVERSION_RATE";

crate::candid_call::impl_candid_call_error!(CmcHostError);

///
/// CmcSource
///
/// Source capability for the certified CMC ICP/XDR conversion rate.
///

pub trait CmcSource: Sync {
    /// Fetch and authenticate the native CMC ICP/XDR conversion rate.
    fn fetch_certified_icp_xdr_rate<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<CmcCertifiedRate, CmcHostError>>;
}

///
/// LiveCmcSource
///
/// Built-in live adapter for the mainnet Cycle Minting Canister.
///

#[derive(Clone, Copy, Debug, Default)]
pub struct LiveCmcSource;

impl CmcSource for LiveCmcSource {
    fn fetch_certified_icp_xdr_rate<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<CmcCertifiedRate, CmcHostError>> {
        Box::pin(async move {
            enforce_mainnet_network(&request.network)?;
            fetch_live_certified_rate(request).await
        })
    }
}

async fn fetch_live_certified_rate(
    request: &CollectionSourceRequest,
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
    let arg = encode_request::<_, CmcHostError>((), "()")?;
    let response = query_encoded::<IcpXdrConversionRateResponse, CmcHostError>(
        &agent,
        &canister,
        GET_ICP_XDR_CONVERSION_RATE_METHOD,
        "IcpXdrConversionRateResponse",
        arg,
    )
    .await?;
    verified_certified_rate(&agent, &canister, response)
}

fn verified_certified_rate(
    agent: &ic_platform::agent::Agent,
    canister: &Principal,
    response: IcpXdrConversionRateResponse,
) -> Result<CmcCertifiedRate, CmcHostError> {
    let hash_tree = authenticate_canister_hash_tree(
        agent,
        canister,
        &response.certificate,
        &response.hash_tree,
        "canister",
    )
    .map_err(map_certified_data_error)?;
    let rate = CmcIcpXdrConversionRate {
        timestamp_seconds: response.data.timestamp_seconds,
        xdr_permyriad_per_icp: response.data.xdr_permyriad_per_icp,
    };
    verify_rate_leaf(&hash_tree, &rate)?;

    Ok(CmcCertifiedRate {
        rate,
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
    hash_tree: &HashTree,
    rate: &CmcIcpXdrConversionRate,
) -> Result<(), CmcHostError> {
    let expected = ic_platform::candid::contract::encode_one(rate).map_err(|error| {
        CmcHostError::CandidEncode {
            message: "IcpXdrConversionRate",
            reason: error.to_string(),
        }
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
mod tests;
