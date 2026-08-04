//! Module: sns::report::model::reports::neurons::detail
//!
//! Responsibility: exact SNS neuron detail and permission-evidence DTOs.
//! Does not own: live Governance calls, SNS discovery, or text rendering.
//! Boundary: preserves variable-size native neuron evidence outside fixed-size list caches.

use super::SnsNeuronRow;
use crate::report::ReportDataSource;
use serde::{Deserialize as SerdeDeserialize, Serialize};

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
/// SnsNeuronPermissionValue
///
/// Raw SNS Governance permission code with its current native label.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronPermissionValue {
    /// Raw integer permission code returned by Governance.
    pub code: i32,
    /// Current native permission label, or `unknown` for an unrecognized code.
    pub name: String,
}

impl SnsNeuronPermissionValue {
    /// Construct one permission value from its raw Governance code.
    #[must_use]
    pub fn from_code(code: i32) -> Self {
        Self {
            code,
            name: sns_neuron_permission_name(code).to_string(),
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
    /// Raw permission codes and current native labels.
    pub permission_types: Vec<SnsNeuronPermissionValue>,
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
    /// Observed status for disabling maturity mint conversions through permissions 7 and 8.
    pub maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus,
    /// Observed status for disabling manual maturity staking through permission 9.
    pub manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus,
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

pub(in crate::sns::report::model) fn neuron_policy_observations(
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
        for code in permission.permission_types.iter().map(|value| value.code) {
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
        0 | 11..=i32::MAX | i32::MIN..=-1 => (
            SnsPolicyObservationStatus::Unassessable,
            SnsPolicyObservationStatus::Unassessable,
        ),
        1..=6 | 10 => (
            SnsPolicyObservationStatus::ObservedSatisfied,
            SnsPolicyObservationStatus::ObservedSatisfied,
        ),
    }
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
    /// Requested IC network identity.
    pub network: String,
    /// SNS-W canister used for targeted discovery.
    pub sns_wasm_canister_id: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// IC API endpoint used for discovery and Governance calls.
    pub source_endpoint: String,
    /// Collector identity recorded in report provenance.
    pub fetched_by: String,
    /// Current SNS-W list position retained as display metadata.
    pub id: usize,
    /// Current SNS name retained as display metadata.
    pub name: String,
    /// Stable SNS Root canister identity.
    pub root_canister_id: String,
    /// Stable SNS Governance canister identity.
    pub governance_canister_id: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_names_cover_native_and_unknown_codes() {
        for (code, expected) in [
            (0, "unspecified"),
            (1, "configure_dissolve_state"),
            (2, "manage_principals"),
            (3, "submit_proposal"),
            (4, "vote"),
            (5, "disburse"),
            (6, "split"),
            (7, "merge_maturity"),
            (8, "disburse_maturity"),
            (9, "stake_maturity"),
            (10, "manage_voting_permission"),
            (11, "unknown"),
            (-1, "unknown"),
        ] {
            assert_eq!(sns_neuron_permission_name(code), expected, "code {code}");
        }
    }

    #[test]
    fn policy_observations_fail_closed_and_prioritize_known_violations() {
        let mut detail = SnsNeuronDetail {
            neuron: SnsNeuronRow {
                neuron_id: "00".repeat(32),
                cached_neuron_stake_e8s: 0,
                maturity_e8s_equivalent: 0,
                staked_maturity_e8s_equivalent: None,
                created_timestamp_seconds: 0,
                created_at: "1970-01-01T00:00:00Z".to_string(),
                source_nns_neuron_id: None,
                auto_stake_maturity: None,
                aging_since_timestamp_seconds: 0,
                dissolve_state: None,
                voting_power_percentage_multiplier: 100,
                vesting_period_seconds: None,
                neuron_fees_e8s: 0,
            },
            permissions: vec![SnsNeuronPermissionRow {
                principal: Some("aaaaa-aa".to_string()),
                permission_types: vec![SnsNeuronPermissionValue::from_code(11)],
            }],
            disburse_maturity_in_progress: Vec::new(),
            followees: Vec::new(),
            topic_followees: None,
            maturity_mint_conversion_observed_disabled:
                SnsPolicyObservationStatus::ObservedSatisfied,
            manual_maturity_staking_observed_disabled:
                SnsPolicyObservationStatus::ObservedSatisfied,
        };

        assert_eq!(
            detail.derived_policy_observations(),
            (
                SnsPolicyObservationStatus::Unassessable,
                SnsPolicyObservationStatus::Unassessable,
            )
        );

        detail.permissions[0]
            .permission_types
            .push(SnsNeuronPermissionValue::from_code(7));
        assert_eq!(
            detail.derived_policy_observations(),
            (
                SnsPolicyObservationStatus::Violated,
                SnsPolicyObservationStatus::Unassessable,
            )
        );
    }
}
