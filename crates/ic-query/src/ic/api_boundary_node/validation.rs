//! Module: ic::api_boundary_node::validation
//!
//! Responsibility: validate and project certified API boundary-node source evidence.
//! Does not own: live `read_state`, public request construction, or rendering.
//! Boundary: labels evidence certified only after source, time, identity, and row validation.

#[cfg(feature = "ic-state-host")]
use super::{
    IC_API_BOUNDARY_NODE_AUTHORITY, IC_API_BOUNDARY_NODE_REPORT_SCHEMA_VERSION,
    IcApiBoundaryNodeHostError, IcApiBoundaryNodeReport, IcApiBoundaryNodeRow,
    IcApiBoundaryNodeSourceData, IcApiBoundaryNodeSourceRequest, IcCertifiedStateProvenance,
    MAX_IC_API_BOUNDARY_NODE_ROWS,
};
#[cfg(feature = "ic-state-host")]
use crate::{
    certification::{CertifiedDataError, validate_certificate_time},
    subnet_catalog::format_utc_timestamp_secs,
};
#[cfg(feature = "ic-state-host")]
use candid::Principal;
#[cfg(feature = "ic-state-host")]
use std::{
    collections::HashSet,
    net::{Ipv4Addr, Ipv6Addr},
};

#[cfg(feature = "ic-state-host")]
pub(in crate::ic) fn report_from_source(
    expected: &IcApiBoundaryNodeSourceRequest,
    mut source: IcApiBoundaryNodeSourceData,
) -> Result<IcApiBoundaryNodeReport, IcApiBoundaryNodeHostError> {
    validate_source(expected, &source.source)?;
    validate_certificate_time(
        expected.observed_at_unix_seconds,
        source.certificate_time_unix_nanos,
    )
    .map_err(map_certificate_time_error)?;
    canonicalize_rows(&mut source.rows)?;

    let certificate_time_unix_seconds = source.certificate_time_unix_nanos / 1_000_000_000;
    Ok(IcApiBoundaryNodeReport {
        provenance: IcCertifiedStateProvenance {
            schema_version: IC_API_BOUNDARY_NODE_REPORT_SCHEMA_VERSION,
            network: source.source.network,
            authority: IC_API_BOUNDARY_NODE_AUTHORITY.to_string(),
            source_endpoint: source.source.endpoint,
            effective_canister_id: source.source.effective_canister_id,
            fetched_at_unix_seconds: source.source.observed_at_unix_seconds,
            fetched_at: source.source.fetched_at,
            fetched_by: source.source.fetched_by,
            certificate_time_unix_nanos: source.certificate_time_unix_nanos,
            certificate_time_unix_seconds,
            certificate_time: format_utc_timestamp_secs(certificate_time_unix_seconds),
            certified: true,
            point_in_time_guaranteed: true,
        },
        node_count: source.rows.len(),
        rows: source.rows,
    })
}

#[cfg(feature = "ic-state-host")]
fn validate_source(
    expected: &IcApiBoundaryNodeSourceRequest,
    actual: &IcApiBoundaryNodeSourceRequest,
) -> Result<(), IcApiBoundaryNodeHostError> {
    if actual != expected {
        return invalid_source(format!(
            "source request is {actual:?}, expected requested value {expected:?}"
        ));
    }
    let expected_fetched_at = format_utc_timestamp_secs(actual.observed_at_unix_seconds);
    if actual.fetched_at != expected_fetched_at {
        return invalid_source(format!(
            "fetched_at is {:?}, expected {expected_fetched_at:?} from observed_at_unix_seconds",
            actual.fetched_at
        ));
    }
    Ok(())
}

#[cfg(feature = "ic-state-host")]
fn canonicalize_rows(rows: &mut [IcApiBoundaryNodeRow]) -> Result<(), IcApiBoundaryNodeHostError> {
    if rows.is_empty() {
        return invalid_source("source returned no API boundary-node rows");
    }
    if rows.len() > MAX_IC_API_BOUNDARY_NODE_ROWS {
        return invalid_source(format!(
            "source returned {} API boundary nodes; maximum is {MAX_IC_API_BOUNDARY_NODE_ROWS}",
            rows.len()
        ));
    }

    let mut node_ids = HashSet::with_capacity(rows.len());
    let mut domains = HashSet::with_capacity(rows.len());
    for row in rows.iter() {
        validate_node_id(&row.node_id)?;
        validate_domain(&row.domain)?;
        if !node_ids.insert(row.node_id.as_str()) {
            return invalid_source(format!("duplicate API boundary-node id {:?}", row.node_id));
        }
        if !domains.insert(row.domain.as_str()) {
            return invalid_source(format!(
                "duplicate API boundary-node domain {:?}",
                row.domain
            ));
        }
        if let Some(ipv4_address) = &row.ipv4_address {
            ipv4_address.parse::<Ipv4Addr>().map_err(|error| {
                invalid_source_value(format!("IPv4 address {ipv4_address:?} is invalid: {error}"))
            })?;
        }
        row.ipv6_address.parse::<Ipv6Addr>().map_err(|error| {
            invalid_source_value(format!(
                "IPv6 address {:?} is invalid: {error}",
                row.ipv6_address
            ))
        })?;
    }

    rows.sort_unstable_by(|left, right| left.node_id.cmp(&right.node_id));
    Ok(())
}

#[cfg(feature = "ic-state-host")]
fn validate_node_id(node_id: &str) -> Result<(), IcApiBoundaryNodeHostError> {
    let principal = Principal::from_text(node_id).map_err(|error| {
        invalid_source_value(format!(
            "API boundary-node id {node_id:?} is invalid: {error}"
        ))
    })?;
    if principal.to_text() != node_id {
        return invalid_source(format!("API boundary-node id {node_id:?} is not canonical"));
    }
    Ok(())
}

#[cfg(feature = "ic-state-host")]
fn validate_domain(domain: &str) -> Result<(), IcApiBoundaryNodeHostError> {
    if domain.is_empty() || domain.len() > 253 || domain != domain.to_ascii_lowercase() {
        return invalid_source(format!(
            "API boundary-node domain {domain:?} must be lowercase DNS text of at most 253 bytes"
        ));
    }
    for label in domain.split('.') {
        let valid = !label.is_empty()
            && label.len() <= 63
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            && label
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric);
        if !valid {
            return invalid_source(format!(
                "API boundary-node domain {domain:?} is not canonical DNS text"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "ic-state-host")]
fn map_certificate_time_error(error: CertifiedDataError) -> IcApiBoundaryNodeHostError {
    match error {
        #[cfg(any(
            feature = "certified-subnet-catalog-host",
            feature = "cmc-host",
            feature = "icrc-host"
        ))]
        CertifiedDataError::Authentication { reason } | CertifiedDataError::Invalid { reason } => {
            invalid_source_value(reason)
        }
        #[cfg(not(any(
            feature = "certified-subnet-catalog-host",
            feature = "cmc-host",
            feature = "icrc-host"
        )))]
        CertifiedDataError::Invalid { reason } => invalid_source_value(reason),
    }
}

#[cfg(feature = "ic-state-host")]
fn invalid_source<T>(reason: impl Into<String>) -> Result<T, IcApiBoundaryNodeHostError> {
    Err(invalid_source_value(reason))
}

#[cfg(feature = "ic-state-host")]
fn invalid_source_value(reason: impl Into<String>) -> IcApiBoundaryNodeHostError {
    IcApiBoundaryNodeHostError::InvalidSourceData {
        reason: reason.into(),
    }
}
