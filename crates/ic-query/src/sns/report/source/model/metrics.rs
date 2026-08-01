//! Module: sns::report::source::model::metrics
//!
//! Responsibility: source result and invariants for bounded SNS metrics collection.
//! Does not own: live transport, lookup, report assembly, or rendering.
//! Boundary: validates target, method, freshness claims, row identity, and bounds.

use crate::sns::report::{
    SnsHostError, SnsTreasuryKind, SnsTreasuryMetricRow, SnsVotingPowerMetrics,
};
use candid::Principal;
use std::collections::BTreeSet;

pub(in crate::sns::report) const SNS_METRICS_METHOD: &str = "get_metrics";
pub(in crate::sns::report) const SNS_METRICS_CALL_TYPE: &str = "composite_query";
const MAX_SNS_TREASURY_METRICS: usize = 16;

///
/// MainnetSnsMetrics
///
/// Source-layer result from one bounded SNS Governance metrics query.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsMetrics {
    /// Governance canister identity queried by the source.
    pub governance_canister_id: String,
    /// Native Governance method queried by the source.
    pub method: String,
    /// Native call type used by the source.
    pub call_type: String,
    /// Proposal-count window supplied to Governance.
    pub time_window_seconds: u64,
    /// Whether the source can prove one point-in-time snapshot.
    pub point_in_time_guaranteed: bool,
    /// Whether treasury values are cached Governance metrics.
    pub treasury_metrics_cached: bool,
    /// Recent submitted-proposal count.
    pub num_recently_submitted_proposals: Option<u64>,
    /// Recent executed-proposal count.
    pub num_recently_executed_proposals: Option<u64>,
    /// Latest SNS-ledger block timestamp observed by Governance.
    pub last_ledger_block_timestamp: Option<u64>,
    /// SNS genesis timestamp.
    pub genesis_timestamp_seconds: Option<u64>,
    /// Cached treasury metrics.
    pub treasury_metrics: Vec<SnsTreasuryMetricRow>,
    /// Cached voting-power metrics.
    pub voting_power_metrics: Option<SnsVotingPowerMetrics>,
}

pub(in crate::sns::report) fn canonicalize_mainnet_sns_metrics(
    metrics: &mut MainnetSnsMetrics,
    expected_governance_canister_id: &str,
    expected_time_window_seconds: u64,
) -> Result<(), SnsHostError> {
    validate_principal("governance_canister_id", &metrics.governance_canister_id)?;
    validate_exact(
        "governance_canister_id",
        expected_governance_canister_id,
        &metrics.governance_canister_id,
    )?;
    validate_exact("method", SNS_METRICS_METHOD, &metrics.method)?;
    validate_exact("call_type", SNS_METRICS_CALL_TYPE, &metrics.call_type)?;
    if metrics.time_window_seconds != expected_time_window_seconds {
        return Err(invalid_metrics(format!(
            "time_window_seconds is {}, expected {expected_time_window_seconds}",
            metrics.time_window_seconds
        )));
    }
    if metrics.point_in_time_guaranteed {
        return Err(invalid_metrics(
            "cached and live metrics cannot claim a point-in-time guarantee".to_string(),
        ));
    }
    if !metrics.treasury_metrics_cached {
        return Err(invalid_metrics(
            "treasury_metrics_cached must identify Governance-cached values".to_string(),
        ));
    }
    if metrics.treasury_metrics.len() > MAX_SNS_TREASURY_METRICS {
        return Err(invalid_metrics(format!(
            "treasury metric count {} exceeds {MAX_SNS_TREASURY_METRICS}",
            metrics.treasury_metrics.len()
        )));
    }

    metrics.treasury_metrics.sort_by_key(|row| row.treasury);
    let mut treasury_codes = BTreeSet::new();
    for row in &metrics.treasury_metrics {
        if !treasury_codes.insert(row.treasury) {
            return Err(invalid_metrics(format!(
                "duplicate treasury code {}",
                row.treasury
            )));
        }
        let expected_kind = sns_treasury_kind(row.treasury);
        if row.treasury_kind != expected_kind {
            return Err(invalid_metrics(format!(
                "treasury code {} has kind {:?}, expected {:?}",
                row.treasury, row.treasury_kind, expected_kind
            )));
        }
        validate_optional_text("treasury name", row.name.as_deref())?;
        validate_optional_principal("ledger_canister_id", row.ledger_canister_id.as_deref())?;
        validate_optional_principal("account_owner", row.account_owner.as_deref())?;
        if let Some(subaccount) = row.account_subaccount_hex.as_deref()
            && (subaccount.len() != 64
                || !subaccount.bytes().all(|byte| byte.is_ascii_hexdigit())
                || subaccount.bytes().any(|byte| byte.is_ascii_uppercase()))
        {
            return Err(invalid_metrics(format!(
                "treasury code {} account_subaccount_hex is not 32-byte lowercase hexadecimal text",
                row.treasury
            )));
        }
    }
    Ok(())
}

pub(in crate::sns::report) const fn sns_treasury_kind(treasury: i32) -> SnsTreasuryKind {
    match treasury {
        0 => SnsTreasuryKind::Unspecified,
        1 => SnsTreasuryKind::Icp,
        2 => SnsTreasuryKind::SnsToken,
        _ => SnsTreasuryKind::Unknown,
    }
}

fn validate_optional_text(field: &'static str, value: Option<&str>) -> Result<(), SnsHostError> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.trim().is_empty() {
        return Err(invalid_metrics(format!("{field} is empty")));
    }
    if value.trim() != value {
        return Err(invalid_metrics(format!(
            "{field} has surrounding whitespace"
        )));
    }
    Ok(())
}

fn validate_optional_principal(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), SnsHostError> {
    if let Some(value) = value {
        validate_principal(field, value)?;
    }
    Ok(())
}

fn validate_principal(field: &'static str, value: &str) -> Result<(), SnsHostError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_metrics(format!("{field} {value:?} is invalid: {error}")))?;
    if principal.to_text() != value {
        return Err(invalid_metrics(format!(
            "{field} {value:?} is not canonical principal text"
        )));
    }
    Ok(())
}

fn validate_exact(field: &'static str, expected: &str, actual: &str) -> Result<(), SnsHostError> {
    if actual != expected {
        return Err(invalid_metrics(format!(
            "{field} is {actual:?}, expected {expected:?}"
        )));
    }
    Ok(())
}

const fn invalid_metrics(reason: String) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: "SNS metrics",
        reason,
    }
}
