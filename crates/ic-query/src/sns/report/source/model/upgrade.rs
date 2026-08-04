//! Module: sns::report::source::model::upgrade
//!
//! Responsibility: source result and invariants for bounded SNS upgrade collection.
//! Does not own: live transport, SNS lookup, report assembly, or rendering.
//! Boundary: validates target identity, native methods, versions, and next-version gaps.

use super::validation::SnsSourceValidator;
use crate::{
    hex::is_canonical_lowercase_hex,
    sns::report::{
        MAINNET_SNS_WASM_CANISTER_ID, SnsCanisterMethod, SnsHostError, SnsPendingUpgrade,
        SnsUpgradeQueryGap, SnsVersion,
    },
};

pub(in crate::sns::report) const SNS_UPGRADE_QUERY_COUNT: usize = 2;
const VALIDATOR: SnsSourceValidator = SnsSourceValidator::new("SNS upgrade");

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
    pub running_version_method: SnsCanisterMethod,
    /// Native SNS-W next-version query method.
    pub next_version_method: SnsCanisterMethod,
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
    VALIDATOR.canonical_principal("governance_canister_id", &upgrade.governance_canister_id)?;
    VALIDATOR.canonical_principal("sns_wasm_canister_id", &upgrade.sns_wasm_canister_id)?;
    VALIDATOR.exact(
        "governance_canister_id",
        expected_governance_canister_id,
        &upgrade.governance_canister_id,
    )?;
    VALIDATOR.exact(
        "sns_wasm_canister_id",
        MAINNET_SNS_WASM_CANISTER_ID,
        &upgrade.sns_wasm_canister_id,
    )?;
    VALIDATOR.exact(
        "running_version_method",
        SnsCanisterMethod::GetRunningSnsVersion.as_str(),
        upgrade.running_version_method.as_str(),
    )?;
    VALIDATOR.exact(
        "next_version_method",
        SnsCanisterMethod::GetNextSnsVersion.as_str(),
        upgrade.next_version_method.as_str(),
    )?;
    if upgrade.point_in_time_guaranteed {
        return Err(VALIDATOR.invalid(
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
            return Err(
                VALIDATOR.invalid("next_version has both a value and a query gap".to_string())
            );
        }
        VALIDATOR.exact(
            "next_version_gap.method",
            SnsCanisterMethod::GetNextSnsVersion.as_str(),
            gap.method.as_str(),
        )?;
        if gap.reason.trim().is_empty() {
            return Err(VALIDATOR.invalid("next_version query gap has an empty reason".to_string()));
        }
        if gap.reason.trim() != gap.reason {
            return Err(VALIDATOR
                .invalid("next_version query gap reason has surrounding whitespace".to_string()));
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
        if !is_canonical_lowercase_hex(hash) {
            return Err(VALIDATOR.invalid(format!(
                "{field}.{role}_wasm_hash_hex is not lowercase even-length hexadecimal text"
            )));
        }
    }
    Ok(())
}
