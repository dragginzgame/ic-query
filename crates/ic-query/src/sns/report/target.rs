//! Explicit Governance targeting without discovery or live lookup.
use crate::{
    report::CollectionSourceRequest,
    sns::report::{SnsHostError, SnsNeuronTargetRequest, enforce_mainnet_network},
};

///
/// ResolvedNeuronTarget
///
/// Validated Governance identity and caller-provided acquisition evidence.
///

pub(in crate::sns::report) struct ResolvedNeuronTarget {
    pub(in crate::sns::report) fetch_request: CollectionSourceRequest,
    pub(in crate::sns::report) governance_canister_id: String,
}
pub(in crate::sns::report) fn resolve_neuron_target(
    request: &SnsNeuronTargetRequest,
) -> Result<ResolvedNeuronTarget, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let governance_canister_id = parse_governance_canister_input(&request.governance_canister_id)?;
    Ok(ResolvedNeuronTarget {
        fetch_request: CollectionSourceRequest::from_unix_secs(
            &request.network,
            &request.source_endpoint,
            request.now_unix_secs,
            "ic-query",
        ),
        governance_canister_id,
    })
}
pub(in crate::sns::report) fn parse_governance_canister_input(
    input: &str,
) -> Result<String, SnsHostError> {
    crate::principal_text::validate_canonical(
        input,
        |reason| SnsHostError::InvalidPrincipal {
            field: "governance_canister_id",
            reason,
        },
        |_| SnsHostError::InvalidPrincipal {
            field: "governance_canister_id",
            reason: "expected canonical principal text".to_string(),
        },
    )?;
    Ok(input.to_string())
}
pub(in crate::sns::report) const fn validate_sns_refresh_page_size(
    page_size: u32,
) -> Result<(), SnsHostError> {
    if page_size == 0 || page_size > super::SNS_REFRESH_MAX_PAGE_SIZE {
        return Err(SnsHostError::InvalidRefreshPageSize {
            page_size,
            max_page_size: super::SNS_REFRESH_MAX_PAGE_SIZE,
        });
    }
    Ok(())
}
