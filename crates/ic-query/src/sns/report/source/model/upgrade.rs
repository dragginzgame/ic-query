//! Module: sns::report::source::model::upgrade
//!
//! Responsibility: source result and invariants for bounded SNS upgrade collection.
//! Does not own: live transport, SNS lookup, report assembly, or rendering.
//! Boundary: validates target identity, native methods, versions, and next-version gaps.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SnsHostError, SnsPendingUpgrade, SnsUpgradeQueryGap, SnsVersion,
};
use candid::Principal;

pub(in crate::sns::report) const SNS_RUNNING_VERSION_METHOD: &str = "get_running_sns_version";
pub(in crate::sns::report) const SNS_NEXT_VERSION_METHOD: &str = "get_next_sns_version";
pub(in crate::sns::report) const SNS_UPGRADE_QUERY_COUNT: usize = 2;

///
/// MainnetSnsUpgrade
///
/// Source-layer result from bounded Governance and SNS-W version queries.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsUpgrade {
    /// Governance canister identity queried by the source.
    pub governance_canister_id: String,
    /// SNS-W canister identity queried by the source.
    pub sns_wasm_canister_id: String,
    /// Native Governance running-version query method.
    pub running_version_method: String,
    /// Native SNS-W next-version query method.
    pub next_version_method: String,
    /// Whether the source can prove one point-in-time snapshot across both queries.
    pub point_in_time_guaranteed: bool,
    /// Governance-reported deployed SNS version.
    pub deployed_version: SnsVersion,
    /// Governance-reported pending upgrade, when present.
    pub pending_upgrade: Option<SnsPendingUpgrade>,
    /// Next blessed SNS-W version, or `None` when no successor exists.
    pub next_version: Option<SnsVersion>,
    /// Failed next-version query retained after deployed-version collection succeeded.
    pub next_version_gap: Option<SnsUpgradeQueryGap>,
}

pub(in crate::sns::report) fn canonicalize_mainnet_sns_upgrade(
    upgrade: &MainnetSnsUpgrade,
    expected_governance_canister_id: &str,
) -> Result<(), SnsHostError> {
    validate_principal("governance_canister_id", &upgrade.governance_canister_id)?;
    validate_principal("sns_wasm_canister_id", &upgrade.sns_wasm_canister_id)?;
    validate_exact(
        "governance_canister_id",
        expected_governance_canister_id,
        &upgrade.governance_canister_id,
    )?;
    validate_exact(
        "sns_wasm_canister_id",
        MAINNET_SNS_WASM_CANISTER_ID,
        &upgrade.sns_wasm_canister_id,
    )?;
    validate_exact(
        "running_version_method",
        SNS_RUNNING_VERSION_METHOD,
        &upgrade.running_version_method,
    )?;
    validate_exact(
        "next_version_method",
        SNS_NEXT_VERSION_METHOD,
        &upgrade.next_version_method,
    )?;
    if upgrade.point_in_time_guaranteed {
        return Err(invalid_upgrade(
            "sequential Governance and SNS-W queries cannot claim a point-in-time guarantee"
                .to_string(),
        ));
    }

    validate_version("deployed_version", &upgrade.deployed_version)?;
    if let Some(pending) = &upgrade.pending_upgrade
        && let Some(version) = &pending.target_version
    {
        validate_version("pending_upgrade.target_version", version)?;
    }
    if let Some(version) = &upgrade.next_version {
        validate_version("next_version", version)?;
    }
    if let Some(gap) = &upgrade.next_version_gap {
        if upgrade.next_version.is_some() {
            return Err(invalid_upgrade(
                "next_version has both a value and a query gap".to_string(),
            ));
        }
        validate_exact(
            "next_version_gap.method",
            SNS_NEXT_VERSION_METHOD,
            &gap.method,
        )?;
        if gap.reason.trim().is_empty() {
            return Err(invalid_upgrade(
                "next_version query gap has an empty reason".to_string(),
            ));
        }
        if gap.reason.trim() != gap.reason {
            return Err(invalid_upgrade(
                "next_version query gap reason has surrounding whitespace".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_version(field: &'static str, version: &SnsVersion) -> Result<(), SnsHostError> {
    for (role, hash) in [
        ("archive", version.archive_wasm_hash_hex.as_str()),
        ("root", version.root_wasm_hash_hex.as_str()),
        ("swap", version.swap_wasm_hash_hex.as_str()),
        ("ledger", version.ledger_wasm_hash_hex.as_str()),
        ("governance", version.governance_wasm_hash_hex.as_str()),
        ("index", version.index_wasm_hash_hex.as_str()),
    ] {
        if hash.is_empty()
            || hash.len() % 2 != 0
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            || hash.bytes().any(|byte| byte.is_ascii_uppercase())
        {
            return Err(invalid_upgrade(format!(
                "{field}.{role}_wasm_hash_hex is not lowercase even-length hexadecimal text"
            )));
        }
    }
    Ok(())
}

fn validate_exact(field: &'static str, expected: &str, actual: &str) -> Result<(), SnsHostError> {
    if actual != expected {
        return Err(invalid_upgrade(format!(
            "{field} is {actual:?}, expected {expected:?}"
        )));
    }
    Ok(())
}

fn validate_principal(field: &'static str, value: &str) -> Result<(), SnsHostError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_upgrade(format!("{field} {value:?} is invalid: {error}")))?;
    if principal.to_text() != value {
        return Err(invalid_upgrade(format!(
            "{field} {value:?} is not canonical principal text"
        )));
    }
    Ok(())
}

const fn invalid_upgrade(reason: String) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: "SNS upgrade",
        reason,
    }
}
