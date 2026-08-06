//! Module: nns::registry::replay::bootstrap
//!
//! Responsibility: run an explicit, resource-reserved Registry bootstrap from version zero.
//! Does not own: persistence, cache policy, catalog projection, or assurance promotion.
//! Boundary: worst-case capacity is checked before every built-in source call.

use super::{NnsRegistryReplayError, NnsRegistryReplaySession, NnsRegistryReplaySessionLimits};
use crate::nns::{
    LiveNnsSource,
    registry::{
        NnsCertifiedRegistryDeltaBatchRequest, NnsCertifiedRegistryDeltaSource,
        NnsRegistryHostError, fetch_nns_certified_registry_delta_batch_with_source_async,
        nns_certified_registry_delta_limits,
    },
};

///
/// NnsCertifiedRegistryBootstrapRequest
///
/// Explicit mainnet source, observation time, and resource limits for complete replay bootstrap.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryBootstrapRequest {
    /// Network identity; only mainnet `ic` is supported.
    pub network: String,
    /// Replica endpoint used for every certified delta request.
    pub source_endpoint: String,
    /// Caller observation time used by every certificate-freshness check.
    pub now_unix_secs: u64,
    /// Cumulative evidence and reconstructed-state ceilings.
    pub limits: NnsRegistryReplaySessionLimits,
}

impl NnsCertifiedRegistryBootstrapRequest {
    /// Create an explicit bootstrap request without selecting default limits.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        limits: NnsRegistryReplaySessionLimits,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            limits,
        }
    }
}

/// Bootstrap complete certified Registry state from the built-in mainnet source.
///
/// This operation may perform multiple sequential queries. It starts at
/// Registry version zero, pins the first response's certified latest version,
/// reserves one worst-case bounded batch before each call, and returns only
/// after that exact target has been reconstructed.
pub async fn bootstrap_nns_certified_registry_async(
    request: &NnsCertifiedRegistryBootstrapRequest,
) -> Result<NnsRegistryReplaySession, NnsRegistryReplayError> {
    bootstrap_nns_certified_registry_with_source_async(request, &LiveNnsSource).await
}

/// Bootstrap complete certified Registry state from an explicit async source.
///
/// Custom sources remain responsible for authenticating their raw certificate
/// evidence and for keeping their internal work within the public batch
/// contract. The coordinator validates every returned report before replay.
pub async fn bootstrap_nns_certified_registry_with_source_async(
    request: &NnsCertifiedRegistryBootstrapRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
) -> Result<NnsRegistryReplaySession, NnsRegistryReplayError> {
    crate::network::enforce_mainnet_network_with(&request.network, |network| {
        NnsRegistryReplayError::InvalidBatch(NnsRegistryHostError::UnsupportedNetwork { network })
    })?;
    let (maximum_batch_query_calls, maximum_batch_response_bytes) = batch_reservation()?;
    let mut session = NnsRegistryReplaySession::new(request.limits);
    loop {
        session.ensure_next_source_call_capacity(
            maximum_batch_query_calls,
            maximum_batch_response_bytes,
        )?;
        let batch_request = NnsCertifiedRegistryDeltaBatchRequest::new(
            &request.network,
            &request.source_endpoint,
            session.state().through_version(),
            request.now_unix_secs,
        );
        let report =
            fetch_nns_certified_registry_delta_batch_with_source_async(&batch_request, source)
                .await?;
        session.apply_batch(&batch_request, &report)?;
        if session.is_complete() {
            return Ok(session);
        }
    }
}

fn batch_reservation() -> Result<(u64, u64), NnsRegistryReplayError> {
    let limits = nns_certified_registry_delta_limits();
    let chunk_calls = u64::try_from(limits.max_chunk_references)
        .map_err(|_| NnsRegistryReplayError::Accounting)?;
    let maximum_batch_query_calls = chunk_calls
        .checked_add(1)
        .ok_or(NnsRegistryReplayError::Accounting)?;
    let maximum_batch_response_bytes = limits
        .max_response_body_bytes
        .checked_add(limits.max_chunk_response_bytes)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(NnsRegistryReplayError::Accounting)?;
    Ok((maximum_batch_query_calls, maximum_batch_response_bytes))
}
