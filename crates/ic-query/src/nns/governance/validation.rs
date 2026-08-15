//! Module: nns::governance::validation
//!
//! Responsibility: validate direct Governance requests, provenance, and payload invariants.
//! Does not own: transport execution, report assembly, or error presentation.
//! Boundary: runs identically for native, canister, and caller-provided sources.

#[cfg(any(all(feature = "canister", target_arch = "wasm32"), test))]
use super::MAX_NNS_GOVERNANCE_RESPONSE_BYTES;
use super::{
    NnsGovernanceError, NnsGovernanceMetrics, NnsGovernanceRequest, NnsGovernanceSourceProvenance,
    NnsGovernanceSourceSelection,
};
use crate::subnet_catalog::{MAINNET_NETWORK, canonical_principal_text};

pub fn validate_governance_request(
    request: &NnsGovernanceRequest,
) -> Result<(), NnsGovernanceError> {
    enforce_mainnet_network(&request.network)?;
    validate_source_selection(&request.source)
}

#[cfg(any(all(feature = "canister", target_arch = "wasm32"), test))]
pub(super) const fn validate_governance_response_size(
    method: &'static str,
    actual_bytes: usize,
) -> Result<(), NnsGovernanceError> {
    if actual_bytes <= MAX_NNS_GOVERNANCE_RESPONSE_BYTES {
        Ok(())
    } else {
        Err(NnsGovernanceError::ResponseTooLarge {
            method,
            actual_bytes,
            maximum_bytes: MAX_NNS_GOVERNANCE_RESPONSE_BYTES,
        })
    }
}

pub fn enforce_mainnet_network(network: &str) -> Result<(), NnsGovernanceError> {
    if network == MAINNET_NETWORK {
        Ok(())
    } else {
        Err(NnsGovernanceError::UnsupportedNetwork {
            network: network.to_string(),
        })
    }
}

fn validate_source_selection(
    selection: &NnsGovernanceSourceSelection,
) -> Result<(), NnsGovernanceError> {
    match selection {
        NnsGovernanceSourceSelection::ReplicaQuery {
            endpoint,
            fetched_by,
        } => validate_replica_query_source(endpoint, fetched_by),
        NnsGovernanceSourceSelection::ReplicatedInterCanisterCall => Ok(()),
    }
}

fn validate_replica_query_source(
    endpoint: &str,
    fetched_by: &str,
) -> Result<(), NnsGovernanceError> {
    if fetched_by.trim().is_empty() {
        return Err(NnsGovernanceError::InvalidSourceSelection {
            reason: "replica_query fetched_by must not be empty".to_string(),
        });
    }
    let Some((scheme, remainder)) = endpoint.split_once("://") else {
        return Err(NnsGovernanceError::InvalidSourceSelection {
            reason: format!("invalid endpoint {endpoint:?}: expected an absolute HTTP(S) URL"),
        });
    };
    if !matches!(scheme, "http" | "https") {
        return Err(NnsGovernanceError::InvalidSourceSelection {
            reason: format!("invalid endpoint {endpoint:?}: expected http or https"),
        });
    }
    if endpoint.contains('?') || endpoint.contains('#') {
        return Err(NnsGovernanceError::InvalidSourceSelection {
            reason: format!("invalid endpoint {endpoint:?}: query and fragment are not allowed"),
        });
    }
    let authority = remainder.split('/').next().unwrap_or_default();
    if authority.is_empty() || authority.chars().any(char::is_whitespace) {
        return Err(NnsGovernanceError::InvalidSourceSelection {
            reason: format!("invalid endpoint {endpoint:?}: a hostname is required"),
        });
    }
    if authority.contains('@') {
        return Err(NnsGovernanceError::InvalidSourceSelection {
            reason: format!("invalid endpoint {endpoint:?}: user information is not allowed"),
        });
    }
    Ok(())
}

pub fn validate_source_provenance(
    selection: &NnsGovernanceSourceSelection,
    provenance: &NnsGovernanceSourceProvenance,
) -> Result<(), NnsGovernanceError> {
    match (selection, provenance) {
        (
            NnsGovernanceSourceSelection::ReplicaQuery {
                endpoint,
                fetched_by,
            },
            NnsGovernanceSourceProvenance::ReplicaQuery {
                endpoint: actual_endpoint,
                fetched_by: actual_fetched_by,
            },
        ) if endpoint == actual_endpoint && fetched_by == actual_fetched_by => Ok(()),
        (
            NnsGovernanceSourceSelection::ReplicatedInterCanisterCall,
            NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
                collector_canister_id,
            },
        ) => {
            let canonical = canonical_principal_text(collector_canister_id).map_err(|error| {
                NnsGovernanceError::SourceEvidenceMismatch {
                    reason: format!("invalid collector_canister_id: {error}"),
                }
            })?;
            if canonical == *collector_canister_id {
                Ok(())
            } else {
                Err(NnsGovernanceError::SourceEvidenceMismatch {
                    reason: "collector_canister_id is not canonical principal text".to_string(),
                })
            }
        }
        _ => Err(NnsGovernanceError::SourceEvidenceMismatch {
            reason: format!("requested {selection:?}, received {provenance:?}"),
        }),
    }
}

/// Validate the network and concrete source retained by a caller-owned Governance report.
pub fn validate_governance_report_source(
    network: &str,
    provenance: &NnsGovernanceSourceProvenance,
) -> Result<(), NnsGovernanceError> {
    enforce_mainnet_network(network)?;
    match provenance {
        NnsGovernanceSourceProvenance::ReplicaQuery {
            endpoint,
            fetched_by,
        } => validate_replica_query_source(endpoint, fetched_by),
        NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall { .. } => {
            validate_source_provenance(
                &NnsGovernanceSourceSelection::ReplicatedInterCanisterCall,
                provenance,
            )
        }
    }
}

pub(super) fn validate_governance_metrics(
    metrics: &NnsGovernanceMetrics,
) -> Result<(), NnsGovernanceError> {
    for (field, buckets) in [
        (
            "not_dissolving_neurons_e8s_buckets",
            metrics.not_dissolving_neurons_e8s_buckets.as_slice(),
        ),
        (
            "dissolving_neurons_staked_maturity_e8s_equivalent_buckets",
            metrics
                .dissolving_neurons_staked_maturity_e8s_equivalent_buckets
                .as_slice(),
        ),
        (
            "not_dissolving_neurons_e8s_buckets_ect",
            metrics.not_dissolving_neurons_e8s_buckets_ect.as_slice(),
        ),
        (
            "dissolving_neurons_e8s_buckets_seed",
            metrics.dissolving_neurons_e8s_buckets_seed.as_slice(),
        ),
        (
            "not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets",
            metrics
                .not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets
                .as_slice(),
        ),
        (
            "dissolving_neurons_e8s_buckets_ect",
            metrics.dissolving_neurons_e8s_buckets_ect.as_slice(),
        ),
        (
            "dissolving_neurons_e8s_buckets",
            metrics.dissolving_neurons_e8s_buckets.as_slice(),
        ),
        (
            "not_dissolving_neurons_e8s_buckets_seed",
            metrics.not_dissolving_neurons_e8s_buckets_seed.as_slice(),
        ),
    ] {
        if let Some(bucket) = buckets.iter().find(|bucket| !bucket.value.is_finite()) {
            return Err(NnsGovernanceError::InvalidMetrics {
                field,
                key: bucket.key,
                value: bucket.value,
            });
        }
    }
    Ok(())
}
