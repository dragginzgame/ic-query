//! Module: sns::report::model::reports::neurons::detail
//!
//! Responsibility: exact SNS neuron detail, permission evidence, and shared evidence validation.
//! Does not own: live Governance calls, SNS discovery, or text rendering.
//! Boundary: preserves variable-size native neuron evidence outside fixed-size list caches.

use super::SnsNeuronRow;
#[cfg(feature = "sns-host")]
use crate::hex::is_lowercase_hex_32_bytes;
use crate::report::ReportDataSource;
use crate::sns::report::SnsReportContext;
use serde::{Deserialize as SerdeDeserialize, Serialize};
#[cfg(feature = "sns-host")]
use std::collections::HashSet;

///
/// SnsPolicyObservationStatus
///
/// Tri-state result for one observed maturity-conversion policy condition.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsPolicyObservationStatus {
    /// Every available value satisfies the observed condition.
    ObservedSatisfied,
    /// At least one available value violates the observed condition.
    Violated,
    /// Unknown or anomalous evidence prevents a closed-world assessment.
    Unassessable,
}

impl SnsPolicyObservationStatus {
    /// Return the stable report label for this status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ObservedSatisfied => "observed_satisfied",
            Self::Violated => "violated",
            Self::Unassessable => "unassessable",
        }
    }

    /// Combine two observations, preserving a known violation over uncertainty.
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Violated, _) | (_, Self::Violated) => Self::Violated,
            (Self::Unassessable, _) | (_, Self::Unassessable) => Self::Unassessable,
            (Self::ObservedSatisfied, Self::ObservedSatisfied) => Self::ObservedSatisfied,
        }
    }
}

///
/// SnsNeuronPermissionRow
///
/// Permissions held by one principal on an SNS neuron.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronPermissionRow {
    /// Canonical principal text when Governance supplied the permission holder.
    pub principal: Option<String>,
    /// Raw Governance permission codes, including unrecognized values.
    pub permission_types: Vec<i32>,
}

///
/// SnsNeuronAccount
///
/// Native optional destination account retained for a pending maturity disbursement.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronAccount {
    /// Canonical destination owner when Governance supplied one.
    pub owner: Option<String>,
    /// Native destination subaccount encoded as lowercase hexadecimal when supplied.
    pub subaccount_hex: Option<String>,
}

///
/// SnsMaturityDisbursementRow
///
/// Native maturity disbursement that Governance has not yet finalized.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsMaturityDisbursementRow {
    /// Unix timestamp at which the maturity disbursement was scheduled.
    pub timestamp_of_disbursement_seconds: u64,
    /// Raw scheduled amount in e8s.
    pub amount_e8s: u64,
    /// Complete optional destination account returned by Governance.
    pub account_to_disburse_to: Option<SnsNeuronAccount>,
    /// Unix timestamp at which Governance expects to finalize the disbursement.
    pub finalize_disbursement_timestamp_seconds: Option<u64>,
}

///
/// SnsNeuronFolloweesRow
///
/// Legacy function-based followees retained from one SNS neuron.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronFolloweesRow {
    /// Native nervous-system function identifier.
    pub function_id: u64,
    /// Full 32-byte followee neuron identifiers encoded as lowercase hexadecimal.
    pub followee_neuron_ids: Vec<String>,
}

///
/// SnsNeuronFolloweeRow
///
/// One native topic-following target and its optional alias.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronFolloweeRow {
    /// Full followee neuron identifier when Governance supplied one.
    pub neuron_id: Option<String>,
    /// Native followee alias when Governance supplied one.
    pub alias: Option<String>,
}

///
/// SnsNeuronTopicFolloweesRow
///
/// Native topic-following entry retained from one SNS neuron.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronTopicFolloweesRow {
    /// Raw topic code used as the native topic-following map key.
    pub topic_code: i32,
    /// Current native topic label when Governance supplied a known topic variant.
    pub topic: Option<String>,
    /// Native topic followees in response order.
    pub followees: Vec<SnsNeuronFolloweeRow>,
}

///
/// SnsNeuronDetail
///
/// Full native detail evidence for exactly one SNS neuron.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronDetail {
    /// Fixed-size neuron state shared with bounded list and snapshot rows.
    pub neuron: SnsNeuronRow,
    /// Every current principal permission entry.
    pub permissions: Vec<SnsNeuronPermissionRow>,
    /// Every pending maturity disbursement.
    pub disburse_maturity_in_progress: Vec<SnsMaturityDisbursementRow>,
    /// Legacy function-based followees.
    pub followees: Vec<SnsNeuronFolloweesRow>,
    /// Topic-based followees when the native optional collection is present.
    pub topic_followees: Option<Vec<SnsNeuronTopicFolloweesRow>>,
}

impl SnsNeuronDetail {
    /// Recompute both neuron-local maturity policy observations from raw evidence.
    #[must_use]
    pub fn derived_policy_observations(
        &self,
    ) -> (SnsPolicyObservationStatus, SnsPolicyObservationStatus) {
        neuron_policy_observations(
            &self.permissions,
            !self.disburse_maturity_in_progress.is_empty(),
        )
    }
}

pub(in crate::sns::report) fn neuron_policy_observations(
    permissions: &[SnsNeuronPermissionRow],
    has_pending_maturity_disbursement: bool,
) -> (SnsPolicyObservationStatus, SnsPolicyObservationStatus) {
    let mut mint = if has_pending_maturity_disbursement {
        SnsPolicyObservationStatus::Violated
    } else {
        SnsPolicyObservationStatus::ObservedSatisfied
    };
    let mut staking = if permissions.is_empty() {
        mint = mint.combine(SnsPolicyObservationStatus::Unassessable);
        SnsPolicyObservationStatus::Unassessable
    } else {
        SnsPolicyObservationStatus::ObservedSatisfied
    };
    for permission in permissions {
        if permission.principal.is_none() || permission.permission_types.is_empty() {
            mint = mint.combine(SnsPolicyObservationStatus::Unassessable);
            staking = staking.combine(SnsPolicyObservationStatus::Unassessable);
        }
        for code in permission.permission_types.iter().copied() {
            let (code_mint, code_staking) = permission_code_policy_observations(code);
            mint = mint.combine(code_mint);
            staking = staking.combine(code_staking);
        }
    }
    (mint, staking)
}

pub(in crate::sns::report::model) const fn permission_code_policy_observations(
    code: i32,
) -> (SnsPolicyObservationStatus, SnsPolicyObservationStatus) {
    match code {
        7 | 8 => (
            SnsPolicyObservationStatus::Violated,
            SnsPolicyObservationStatus::ObservedSatisfied,
        ),
        9 => (
            SnsPolicyObservationStatus::ObservedSatisfied,
            SnsPolicyObservationStatus::Violated,
        ),
        _ if permission_code_is_assessable(code) => (
            SnsPolicyObservationStatus::ObservedSatisfied,
            SnsPolicyObservationStatus::ObservedSatisfied,
        ),
        _ => (
            SnsPolicyObservationStatus::Unassessable,
            SnsPolicyObservationStatus::Unassessable,
        ),
    }
}

pub(in crate::sns::report::model) const fn permission_code_is_assessable(code: i32) -> bool {
    matches!(code, 1..=10)
}

///
/// SnsNeuronDetailReport
///
/// Serializable live report for one exact SNS Governance neuron lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronDetailReport {
    /// Report schema version.
    pub schema_version: u32,
    /// SNS identity and provenance, serialized as flat report fields.
    #[serde(flatten)]
    pub context: SnsReportContext,
    /// Exact requested neuron identifier.
    pub neuron_id: String,
    /// Explicit report data source; exact detail reports are live-only.
    pub data_source: ReportDataSource,
    /// Full native neuron detail and derived policy observations.
    pub detail: SnsNeuronDetail,
}

/// Return the current native label for one raw SNS neuron permission code.
#[must_use]
pub const fn sns_neuron_permission_name(code: i32) -> &'static str {
    match code {
        0 => "unspecified",
        1 => "configure_dissolve_state",
        2 => "manage_principals",
        3 => "submit_proposal",
        4 => "vote",
        5 => "disburse",
        6 => "split",
        7 => "merge_maturity",
        8 => "disburse_maturity",
        9 => "stake_maturity",
        10 => "manage_voting_permission",
        _ => "unknown",
    }
}

#[cfg(feature = "sns-host")]
pub(in crate::sns::report) fn validate_neuron_permissions(
    permissions: &[SnsNeuronPermissionRow],
) -> Result<(), String> {
    let mut principals = HashSet::new();
    for permission in permissions {
        let principal = match permission.principal.as_deref() {
            Some(principal) => {
                validate_canonical_principal(principal, "permission principal")?;
                if !principals.insert(principal) {
                    return Err(format!("duplicate permission principal {principal}"));
                }
                principal
            }
            None => {
                return Err("permission principal is missing".to_string());
            }
        };
        let mut codes = HashSet::new();
        for code in &permission.permission_types {
            if !codes.insert(*code) {
                return Err(format!(
                    "{principal} contains duplicate permission code {code}"
                ));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "sns-host")]
pub(in crate::sns::report) fn validate_maturity_disbursements(
    disbursements: &[SnsMaturityDisbursementRow],
) -> Result<(), String> {
    for disbursement in disbursements {
        if let Some(account) = disbursement.account_to_disburse_to.as_ref() {
            if let Some(owner) = account.owner.as_deref() {
                validate_canonical_principal(owner, "pending disbursement account owner")?;
            }
            if let Some(subaccount) = account.subaccount_hex.as_deref()
                && !is_lowercase_hex_32_bytes(subaccount)
            {
                return Err(
                    "pending disbursement subaccount is not 32-byte lowercase hexadecimal text"
                        .to_string(),
                );
            }
        }
    }
    Ok(())
}

#[cfg(feature = "sns-host")]
pub(super) fn validate_canonical_principal(value: &str, field: &str) -> Result<(), String> {
    crate::principal_text::validate_canonical(
        value,
        |reason| format!("{field} {value} is invalid: {reason}"),
        |_| format!("{field} {value} is not canonical principal text"),
    )
}

#[cfg(test)]
mod tests;
