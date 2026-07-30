//! Module: ic::source
//!
//! Responsibility: IC Dashboard source contract, validation, and canonical projection.
//! Does not own: HTTP transport, command parsing, or text rendering.
//! Boundary: treats live and custom source results as untrusted authority data.

use crate::ic::{
    IC_CANISTER_REPORT_SCHEMA_VERSION, IC_DASHBOARD_AUTHORITY, IC_DASHBOARD_NETWORK,
    IcCanisterReport, IcCanisterSourceData, IcCanisterUpgrade, IcHostError, IcSourceRequest,
};
use candid::Principal;
use std::collections::HashSet;

///
/// IcCanisterSource
///
/// Source contract for fetching one canister from an IC Dashboard-compatible API.
///

pub trait IcCanisterSource {
    /// Fetch one canister with explicit endpoint and collection provenance.
    fn fetch_canister(
        &self,
        request: &IcSourceRequest,
        canister_id: &str,
    ) -> Result<IcCanisterSourceData, IcHostError>;
}

pub(super) fn canonical_canister_id(value: &str) -> Result<String, IcHostError> {
    Principal::from_text(value)
        .map(|principal| principal.to_text())
        .map_err(|error| IcHostError::InvalidPrincipal {
            field: "canister_id",
            reason: error.to_string(),
        })
}

pub(super) fn report_from_source(
    request: &IcSourceRequest,
    requested_canister_id: &str,
    mut source: IcCanisterSourceData,
) -> Result<IcCanisterReport, IcHostError> {
    validate_provenance(request, &source)?;
    validate_principal_match("canister_id", requested_canister_id, &source.canister_id)?;
    validate_canonical_principal("subnet_id", &source.subnet_id)?;

    let mut seen_controllers = HashSet::with_capacity(source.controllers.len());
    for controller in &source.controllers {
        validate_canonical_principal("controller", controller)?;
        if !seen_controllers.insert(controller.clone()) {
            return invalid_source(format!("duplicate controller principal {controller}"));
        }
    }
    source.controllers.sort_unstable();

    validate_optional_module_hash("module_hash", &source.module_hash)?;
    if source.dashboard_updated_at.is_empty() {
        return invalid_source("dashboard_updated_at must not be empty");
    }

    if let Some(upgrades) = source.upgrades.as_mut() {
        validate_upgrades(upgrades)?;
        upgrades.sort_unstable_by(|left, right| {
            right
                .executed_timestamp_seconds
                .cmp(&left.executed_timestamp_seconds)
                .then_with(|| right.proposal_id.cmp(&left.proposal_id))
                .then_with(|| left.module_hash.cmp(&right.module_hash))
        });
    }

    Ok(IcCanisterReport {
        schema_version: IC_CANISTER_REPORT_SCHEMA_VERSION,
        network: IC_DASHBOARD_NETWORK.to_string(),
        authority: IC_DASHBOARD_AUTHORITY.to_string(),
        source_endpoint: source.source_endpoint,
        fetched_at: source.fetched_at,
        fetched_by: source.fetched_by,
        certified: false,
        point_in_time_guaranteed: false,
        canister_id: source.canister_id,
        dashboard_id: source.dashboard_id,
        canister_type: source.canister_type,
        name: source.name,
        subnet_id: source.subnet_id,
        controllers: source.controllers,
        language: source.language,
        module_hash: source.module_hash,
        dashboard_updated_at: source.dashboard_updated_at,
        upgrade_count: source.upgrades.as_ref().map(Vec::len),
        upgrades: source.upgrades,
    })
}

fn validate_provenance(
    request: &IcSourceRequest,
    source: &IcCanisterSourceData,
) -> Result<(), IcHostError> {
    for (field, expected, actual) in [
        (
            "source_endpoint",
            request.endpoint.as_str(),
            source.source_endpoint.as_str(),
        ),
        (
            "fetched_at",
            request.fetched_at.as_str(),
            source.fetched_at.as_str(),
        ),
        (
            "fetched_by",
            request.fetched_by.as_str(),
            source.fetched_by.as_str(),
        ),
    ] {
        if actual != expected {
            return invalid_source(format!(
                "{field} is {actual:?}, expected requested value {expected:?}"
            ));
        }
    }
    Ok(())
}

fn validate_principal_match(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), IcHostError> {
    validate_canonical_principal(field, actual)?;
    if actual == expected {
        return Ok(());
    }
    invalid_source(format!(
        "{field} is {actual:?}, expected requested principal {expected:?}"
    ))
}

fn validate_canonical_principal(field: &'static str, value: &str) -> Result<(), IcHostError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_source_value(format!("{field} {value:?}: {error}")))?;
    let canonical = principal.to_text();
    if canonical != value {
        return invalid_source(format!(
            "{field} {value:?} is not canonical principal text; expected {canonical:?}"
        ));
    }
    Ok(())
}

fn validate_upgrades(upgrades: &[IcCanisterUpgrade]) -> Result<(), IcHostError> {
    let mut proposal_ids = HashSet::with_capacity(upgrades.len());
    for upgrade in upgrades {
        validate_module_hash("upgrade.module_hash", &upgrade.module_hash)?;
        if !proposal_ids.insert(upgrade.proposal_id) {
            return invalid_source(format!(
                "duplicate upgrade proposal_id {}",
                upgrade.proposal_id
            ));
        }
    }
    Ok(())
}

fn validate_optional_module_hash(field: &'static str, value: &str) -> Result<(), IcHostError> {
    if value.is_empty() {
        return Ok(());
    }
    validate_module_hash(field, value)
}

fn validate_module_hash(field: &'static str, value: &str) -> Result<(), IcHostError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    invalid_source(format!(
        "{field} must be 64-character lowercase hexadecimal text"
    ))
}

fn invalid_source<T>(reason: impl Into<String>) -> Result<T, IcHostError> {
    Err(invalid_source_value(reason))
}

fn invalid_source_value(reason: impl Into<String>) -> IcHostError {
    IcHostError::InvalidSourceData {
        reason: reason.into(),
    }
}
