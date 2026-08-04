//! Module: sns::report::model::reports::neurons::reward
//!
//! Responsibility: SNS reward-event and API-exhausted maturity checkpoint DTOs.
//! Does not own: live Governance calls, strict pagination, or filesystem loading.
//! Boundary: retains raw bracketing evidence, variable neuron evidence, and recomputed summaries.

use super::{
    SnsMaturityDisbursementRow, SnsNeuronPermissionRow,
    detail::{
        SnsPolicyObservationStatus, neuron_policy_observations, permission_code_policy_observations,
    },
};
use crate::{
    hex::{is_canonical_lowercase_hex, is_lowercase_hex},
    report::ReportDataSource,
    sns::report::{
        MAINNET_SNS_WASM_CANISTER_ID, SNS_REWARD_CHECKPOINT_MAX_NEURONS,
        SNS_REWARD_CHECKPOINT_PAGE_SIZE, SNS_REWARD_CHECKPOINT_REPORT_SCHEMA_VERSION,
        model::reports::{SnsGovernanceParameters, SnsRunningVersionResponse},
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashSet;
use thiserror::Error as ThisError;

///
/// SnsRewardProposalId
///
/// Native proposal identifier retained in one Governance reward event.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsRewardProposalId {
    /// Native SNS Governance proposal identifier.
    pub id: u64,
}

///
/// SnsRewardEvent
///
/// Complete native SNS Governance reward-event response.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsRewardEvent {
    /// Number of reward rounds represented by this event when supplied.
    pub rounds_since_last_distribution: Option<u64>,
    /// Native timestamp at which distribution actually ran.
    pub actual_timestamp_seconds: u64,
    /// Canonical reward-event position when supplied.
    pub end_timestamp_seconds: Option<u64>,
    /// Total maturity available for distribution when supplied.
    pub total_available_e8s_equivalent: Option<u64>,
    /// Exact maturity amount distributed by this event.
    pub distributed_e8s_equivalent: u64,
    /// Deprecated native reward round retained as continuity evidence.
    pub round: u64,
    /// Complete settled-proposal identifiers retained from Governance.
    pub settled_proposals: Vec<SnsRewardProposalId>,
}

///
/// SnsRewardCollectionStatus
///
/// Authority level of one completed SNS reward checkpoint collection.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsRewardCollectionStatus {
    /// The native neuron API was exhausted with stable bracket responses.
    ApiExhaustedObserved,
}

impl SnsRewardCollectionStatus {
    /// Return the stable report label for this collection status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApiExhaustedObserved => "api_exhausted_observed",
        }
    }
}

///
/// SnsRewardCheckpointRow
///
/// Variable-size maturity and permission evidence for one observed SNS neuron.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsRewardCheckpointRow {
    /// Exact 32-byte neuron identifier as lowercase hexadecimal text.
    pub neuron_id: String,
    /// Unix timestamp at which Governance reports the neuron was created.
    pub created_timestamp_seconds: u64,
    /// Raw unstaked maturity in e8s-equivalent units.
    pub maturity_e8s_equivalent: u64,
    /// Raw staked maturity in e8s-equivalent units when present.
    pub staked_maturity_e8s_equivalent: Option<u64>,
    /// Checked sum of unstaked and staked maturity.
    pub combined_maturity_e8s_equivalent: u64,
    /// Native automatic maturity-staking state when supplied.
    pub auto_stake_maturity: Option<bool>,
    /// Every current principal permission entry returned by Governance.
    pub permissions: Vec<SnsNeuronPermissionRow>,
    /// Every pending maturity disbursement returned by Governance.
    pub disburse_maturity_in_progress: Vec<SnsMaturityDisbursementRow>,
    /// Neuron-local observation for permissions 7 and 8 plus pending disbursements.
    pub maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus,
    /// Neuron-local observation for manual maturity-staking permission 9.
    pub manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus,
}

impl SnsRewardCheckpointRow {
    /// Recompute combined maturity from the two raw maturity components.
    #[must_use]
    pub fn checked_combined_maturity(&self) -> Option<u64> {
        self.maturity_e8s_equivalent
            .checked_add(self.staked_maturity_e8s_equivalent.unwrap_or(0))
    }

    /// Recompute both row-local maturity policy observations from raw evidence.
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

///
/// SnsRewardCheckpointReport
///
/// Versioned API-exhausted observed SNS maturity checkpoint.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsRewardCheckpointReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Requested IC network identity.
    pub network: String,
    /// Stable mainnet SNS-W discovery canister identity.
    pub sns_wasm_canister_id: String,
    /// Explicit IC API endpoint used for every live call.
    pub source_endpoint: String,
    /// Collector identity recorded in report provenance.
    pub fetched_by: String,
    /// Current SNS-W list position retained only as display metadata.
    pub id: usize,
    /// Current SNS name retained only as display metadata.
    pub name: String,
    /// Stable SNS Root canister identity.
    pub root_canister_id: String,
    /// Stable SNS Governance canister identity.
    pub governance_canister_id: String,
    /// Stable SNS ledger canister identity.
    pub ledger_canister_id: String,
    /// Stable SNS decentralization-swap canister identity.
    pub swap_canister_id: String,
    /// Stable SNS ledger-index canister identity.
    pub index_canister_id: String,
    /// Explicit source classification; checkpoint reports are live-only.
    pub data_source: ReportDataSource,
    /// Unix timestamp captured immediately before targeted discovery.
    pub collection_started_at_unix_secs: u64,
    /// UTC rendering of `collection_started_at_unix_secs`.
    pub collection_started_at: String,
    /// Unix timestamp captured after the final running-version bracket.
    pub collection_completed_at_unix_secs: u64,
    /// UTC rendering of `collection_completed_at_unix_secs`.
    pub collection_completed_at: String,
    /// Fixed native page size used for every Governance neuron query.
    pub page_size: u32,
    /// Number of neuron pages fetched, including the final exhaustion page.
    pub page_count: u32,
    /// Number of neuron rows retained.
    pub row_count: usize,
    /// Recomputed number of unique full neuron identifiers.
    pub unique_neuron_id_count: usize,
    /// Mandatory row ceiling derived from Governance parameters.
    pub collection_row_ceiling: u64,
    /// Total targeted-discovery, bracket, and neuron-page client queries.
    pub client_query_count: u32,
    /// Explicit authority classification for the completed collection.
    pub collection_status: SnsRewardCollectionStatus,
    /// Always false because Governance exposes no point-in-time neuron snapshot version.
    pub point_in_time_guaranteed: bool,
    /// Complete nervous-system parameters read before neuron pagination.
    pub parameters_before: SnsGovernanceParameters,
    /// Complete nervous-system parameters read after neuron pagination.
    pub parameters_after: SnsGovernanceParameters,
    /// Complete reward event read before neuron pagination.
    pub reward_event_before: SnsRewardEvent,
    /// Complete reward event read after neuron pagination.
    pub reward_event_after: SnsRewardEvent,
    /// Complete running-version response read before neuron pagination.
    pub running_version_before: SnsRunningVersionResponse,
    /// Complete running-version response read after neuron pagination.
    pub running_version_after: SnsRunningVersionResponse,
    /// Checked aggregate unstaked maturity across every row.
    pub aggregate_maturity_e8s_equivalent: u64,
    /// Checked aggregate staked maturity across every row.
    pub aggregate_staked_maturity_e8s_equivalent: u64,
    /// Checked aggregate combined maturity across every row.
    pub aggregate_combined_maturity_e8s_equivalent: u64,
    /// Number of principal permission entries observed across every row.
    pub permission_entry_count: usize,
    /// Number of raw unknown or anomalous permission codes observed globally and per-neuron.
    pub unassessable_permission_code_count: usize,
    /// Number of pending maturity disbursements observed across every row.
    pub pending_maturity_disbursement_count: usize,
    /// Number of neurons with automatic maturity staking enabled.
    pub auto_stake_maturity_enabled_count: usize,
    /// Number of neurons with automatic maturity staking explicitly disabled.
    pub auto_stake_maturity_disabled_count: usize,
    /// Number of neurons without an automatic maturity-staking value.
    pub auto_stake_maturity_unspecified_count: usize,
    /// Whether permission 2 is currently grantable, or unknown when the list is missing.
    pub manage_principals_grantable: Option<bool>,
    /// Global observation for disabling permissions 7 and 8 and pending disbursements.
    pub maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus,
    /// Global observation for disabling manual maturity-staking permission 9.
    pub manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus,
    /// Combined observed maturity-conversion policy status.
    pub maturity_conversion_policy_observed_status: SnsPolicyObservationStatus,
    /// Strictly increasing full neuron rows retained as raw checkpoint evidence.
    pub rows: Vec<SnsRewardCheckpointRow>,
}

///
/// SnsRewardCheckpointValidationError
///
/// Pure validation failure for untrusted serialized or in-memory checkpoint evidence.
///

#[derive(Debug, Eq, PartialEq, ThisError)]
#[error("invalid SNS reward checkpoint: {reason}")]
pub struct SnsRewardCheckpointValidationError {
    /// Deterministic invariant failure.
    pub reason: String,
}

/// Recompute and validate every checkpoint invariant available without live host calls.
pub fn validate_sns_reward_checkpoint_report(
    report: &SnsRewardCheckpointReport,
) -> Result<(), SnsRewardCheckpointValidationError> {
    validate_checkpoint_header(report)?;
    validate_checkpoint_brackets(report)?;
    validate_checkpoint_rows(&report.rows, report.collection_completed_at_unix_secs)?;
    let summary = recompute_reward_checkpoint_summary(&report.parameters_before, &report.rows)
        .map_err(invalid_validation)?;
    validate_checkpoint_summary(report, &summary)
}

fn validate_checkpoint_header(
    report: &SnsRewardCheckpointReport,
) -> Result<(), SnsRewardCheckpointValidationError> {
    if report.schema_version != SNS_REWARD_CHECKPOINT_REPORT_SCHEMA_VERSION {
        return Err(invalid_validation(format!(
            "schema version {} does not equal {}",
            report.schema_version, SNS_REWARD_CHECKPOINT_REPORT_SCHEMA_VERSION
        )));
    }
    for (field, principal) in [
        ("sns_wasm_canister_id", report.sns_wasm_canister_id.as_str()),
        ("root_canister_id", report.root_canister_id.as_str()),
        (
            "governance_canister_id",
            report.governance_canister_id.as_str(),
        ),
        ("ledger_canister_id", report.ledger_canister_id.as_str()),
        ("swap_canister_id", report.swap_canister_id.as_str()),
        ("index_canister_id", report.index_canister_id.as_str()),
    ] {
        validate_principal(field, principal)?;
    }
    let canister_ids = [
        report.sns_wasm_canister_id.as_str(),
        report.root_canister_id.as_str(),
        report.governance_canister_id.as_str(),
        report.ledger_canister_id.as_str(),
        report.swap_canister_id.as_str(),
        report.index_canister_id.as_str(),
    ];
    if canister_ids.into_iter().collect::<HashSet<_>>().len() != canister_ids.len() {
        return Err(invalid_validation(
            "checkpoint canister roles do not contain unique principals",
        ));
    }
    if report.sns_wasm_canister_id != MAINNET_SNS_WASM_CANISTER_ID {
        return Err(invalid_validation("unexpected SNS-W canister identity"));
    }
    if report.network != MAINNET_NETWORK {
        return Err(invalid_validation("checkpoint network must be ic"));
    }
    if report.source_endpoint.is_empty() || report.fetched_by.is_empty() {
        return Err(invalid_validation(
            "checkpoint source_endpoint and fetched_by must be non-empty",
        ));
    }
    if report.id == 0 || report.name.is_empty() {
        return Err(invalid_validation(
            "checkpoint SNS list id and display name must be non-empty",
        ));
    }
    if report.data_source != ReportDataSource::Live {
        return Err(invalid_validation("checkpoint data_source must be live"));
    }
    if report.collection_status != SnsRewardCollectionStatus::ApiExhaustedObserved {
        return Err(invalid_validation(
            "checkpoint collection is not API-exhausted",
        ));
    }
    if report.point_in_time_guaranteed {
        return Err(invalid_validation(
            "checkpoint cannot claim a point-in-time neuron snapshot",
        ));
    }
    validate_checkpoint_collection_metadata(report)
}

fn validate_checkpoint_collection_metadata(
    report: &SnsRewardCheckpointReport,
) -> Result<(), SnsRewardCheckpointValidationError> {
    if report.collection_completed_at_unix_secs < report.collection_started_at_unix_secs {
        return Err(invalid_validation(
            "collection completion precedes collection start",
        ));
    }
    if report.collection_started_at
        != format_utc_timestamp_secs(report.collection_started_at_unix_secs)
        || report.collection_completed_at
            != format_utc_timestamp_secs(report.collection_completed_at_unix_secs)
    {
        return Err(invalid_validation(
            "collection UTC timestamps do not match raw Unix timestamps",
        ));
    }
    if report.page_size != SNS_REWARD_CHECKPOINT_PAGE_SIZE || report.page_count == 0 {
        return Err(invalid_validation(
            "checkpoint must contain at least one fixed-size-100 page",
        ));
    }
    let expected_query_count = report
        .page_count
        .checked_add(8)
        .ok_or_else(|| invalid_validation("page_count + 8 exceeds the native u32 contract"))?;
    if report.client_query_count != expected_query_count {
        return Err(invalid_validation(
            "client_query_count does not equal page_count + 8",
        ));
    }
    if report.row_count != report.rows.len() || report.unique_neuron_id_count != report.rows.len() {
        return Err(invalid_validation(
            "serialized row counts do not match raw checkpoint rows",
        ));
    }
    let max_neurons = report.parameters_before.max_number_of_neurons;
    if max_neurons != Some(report.collection_row_ceiling)
        || !(1..=SNS_REWARD_CHECKPOINT_MAX_NEURONS).contains(&report.collection_row_ceiling)
    {
        return Err(invalid_validation(
            "collection_row_ceiling does not match a valid max_number_of_neurons",
        ));
    }
    if u64::try_from(report.rows.len())
        .ok()
        .is_none_or(|rows| rows > report.collection_row_ceiling)
    {
        return Err(invalid_validation(
            "raw row count exceeds the mandatory collection ceiling",
        ));
    }
    Ok(())
}

fn validate_checkpoint_brackets(
    report: &SnsRewardCheckpointReport,
) -> Result<(), SnsRewardCheckpointValidationError> {
    if report.parameters_before != report.parameters_after {
        return Err(invalid_validation(
            "nervous-system parameter brackets differ",
        ));
    }
    if report.reward_event_before != report.reward_event_after {
        return Err(invalid_validation("reward-event brackets differ"));
    }
    if report.running_version_before != report.running_version_after {
        return Err(invalid_validation("running-version brackets differ"));
    }
    validate_sns_reward_event_evidence(&report.reward_event_after).map_err(invalid_validation)?;
    validate_reward_event_position(report)?;
    validate_sns_reward_running_version_evidence(&report.running_version_after)
        .map_err(invalid_validation)?;
    validate_sns_reward_checkpoint_parameter_evidence(&report.parameters_after)
        .map_err(invalid_validation)
}

fn validate_reward_event_position(
    report: &SnsRewardCheckpointReport,
) -> Result<(), SnsRewardCheckpointValidationError> {
    let event = &report.reward_event_after;
    if event.actual_timestamp_seconds > report.collection_completed_at_unix_secs
        || event
            .end_timestamp_seconds
            .is_none_or(|end| end > report.collection_completed_at_unix_secs)
    {
        return Err(invalid_validation(
            "reward event timestamps exceed checkpoint collection completion",
        ));
    }
    Ok(())
}

fn validate_checkpoint_rows(
    rows: &[SnsRewardCheckpointRow],
    collection_completed_at_unix_secs: u64,
) -> Result<(), SnsRewardCheckpointValidationError> {
    let mut previous_id: Option<&str> = None;
    for row in rows {
        if row.neuron_id.len() != 64 || !is_canonical_lowercase_hex(&row.neuron_id) {
            return Err(invalid_validation(format!(
                "neuron id {} is not 32-byte lowercase hexadecimal text",
                row.neuron_id
            )));
        }
        if previous_id.is_some_and(|previous| previous >= row.neuron_id.as_str()) {
            return Err(invalid_validation(
                "checkpoint neuron ids are not strictly increasing",
            ));
        }
        previous_id = Some(&row.neuron_id);
        if row.created_timestamp_seconds > collection_completed_at_unix_secs {
            return Err(invalid_validation(format!(
                "neuron {} creation timestamp exceeds collection completion",
                row.neuron_id
            )));
        }
        validate_checkpoint_row_evidence(row)?;
    }
    Ok(())
}

fn validate_checkpoint_row_evidence(
    row: &SnsRewardCheckpointRow,
) -> Result<(), SnsRewardCheckpointValidationError> {
    let mut principals = HashSet::new();
    for permission in &row.permissions {
        if let Some(principal) = permission.principal.as_deref() {
            validate_principal("permission principal", principal)?;
            if !principals.insert(principal) {
                return Err(invalid_validation(format!(
                    "neuron {} contains duplicate permission principal {principal}",
                    row.neuron_id
                )));
            }
        }
        let mut codes = HashSet::new();
        for value in &permission.permission_types {
            if value.name != super::sns_neuron_permission_name(value.code)
                || !codes.insert(value.code)
            {
                return Err(invalid_validation(format!(
                    "neuron {} contains invalid or duplicate permission code {}",
                    row.neuron_id, value.code
                )));
            }
        }
    }
    for disbursement in &row.disburse_maturity_in_progress {
        if let Some(account) = disbursement.account_to_disburse_to.as_ref() {
            if let Some(owner) = account.owner.as_deref() {
                validate_principal("pending disbursement owner", owner)?;
            }
            if let Some(subaccount) = account.subaccount_hex.as_deref()
                && (subaccount.len() != 64 || !is_lowercase_hex(subaccount))
            {
                return Err(invalid_validation(
                    "pending disbursement subaccount is not 32-byte lowercase hexadecimal text",
                ));
            }
        }
    }
    Ok(())
}

fn validate_checkpoint_summary(
    report: &SnsRewardCheckpointReport,
    summary: &SnsRewardCheckpointSummary,
) -> Result<(), SnsRewardCheckpointValidationError> {
    let valid = report.aggregate_maturity_e8s_equivalent
        == summary.aggregate_maturity_e8s_equivalent
        && report.aggregate_staked_maturity_e8s_equivalent
            == summary.aggregate_staked_maturity_e8s_equivalent
        && report.aggregate_combined_maturity_e8s_equivalent
            == summary.aggregate_combined_maturity_e8s_equivalent
        && report.permission_entry_count == summary.permission_entry_count
        && report.unassessable_permission_code_count == summary.unassessable_permission_code_count
        && report.pending_maturity_disbursement_count
            == summary.pending_maturity_disbursement_count
        && report.auto_stake_maturity_enabled_count == summary.auto_stake_maturity_enabled_count
        && report.auto_stake_maturity_disabled_count == summary.auto_stake_maturity_disabled_count
        && report.auto_stake_maturity_unspecified_count
            == summary.auto_stake_maturity_unspecified_count
        && report.manage_principals_grantable == summary.manage_principals_grantable
        && report.maturity_mint_conversion_observed_disabled
            == summary.maturity_mint_conversion_observed_disabled
        && report.manual_maturity_staking_observed_disabled
            == summary.manual_maturity_staking_observed_disabled
        && report.maturity_conversion_policy_observed_status
            == summary.maturity_conversion_policy_observed_status;
    if valid {
        Ok(())
    } else {
        Err(invalid_validation(
            "serialized checkpoint summary does not match raw evidence",
        ))
    }
}

pub(in crate::sns::report) fn validate_sns_reward_running_version_evidence(
    response: &SnsRunningVersionResponse,
) -> Result<(), String> {
    let deployed = response
        .deployed_version
        .as_ref()
        .ok_or_else(|| "running version has no deployed_version".to_string())?;
    validate_version("deployed_version", deployed)?;
    if let Some(target) = response
        .pending_version
        .as_ref()
        .and_then(|pending| pending.target_version.as_ref())
    {
        validate_version("pending_version.target_version", target)?;
    }
    Ok(())
}

fn validate_version(field: &str, version: &crate::sns::report::SnsVersion) -> Result<(), String> {
    for (role, hash) in [
        ("archive", version.archive_wasm_hash_hex.as_str()),
        ("root", version.root_wasm_hash_hex.as_str()),
        ("swap", version.swap_wasm_hash_hex.as_str()),
        ("ledger", version.ledger_wasm_hash_hex.as_str()),
        ("governance", version.governance_wasm_hash_hex.as_str()),
        ("index", version.index_wasm_hash_hex.as_str()),
    ] {
        if hash.len() != 64 || !is_canonical_lowercase_hex(hash) {
            return Err(format!(
                "{field}.{role}_wasm_hash_hex is not a 32-byte lowercase hexadecimal hash"
            ));
        }
    }
    Ok(())
}

pub(in crate::sns::report) fn validate_sns_reward_checkpoint_parameter_evidence(
    parameters: &SnsGovernanceParameters,
) -> Result<(), String> {
    validate_default_followees(parameters)?;
    validate_parameter_permissions(parameters)
}

fn validate_default_followees(parameters: &SnsGovernanceParameters) -> Result<(), String> {
    let Some(defaults) = parameters.default_followees.as_ref() else {
        return Ok(());
    };
    let mut function_ids = HashSet::new();
    for row in &defaults.followees {
        if !function_ids.insert(row.function_id) {
            return Err("default followees contain a duplicate function id".to_string());
        }
        let mut neuron_ids = HashSet::new();
        for id in &row.followee_neuron_ids {
            if id.len() != 64 || !is_lowercase_hex(id) {
                return Err("default followees contain a non-canonical neuron id".to_string());
            }
            if !neuron_ids.insert(id) {
                return Err("default followees contain a duplicate neuron id".to_string());
            }
        }
    }
    Ok(())
}

fn validate_parameter_permissions(parameters: &SnsGovernanceParameters) -> Result<(), String> {
    for (field, permissions) in [
        (
            "neuron_claimer_permissions",
            parameters.neuron_claimer_permissions.as_ref(),
        ),
        (
            "neuron_grantable_permissions",
            parameters.neuron_grantable_permissions.as_ref(),
        ),
    ] {
        let Some(permissions) = permissions else {
            continue;
        };
        let mut codes = HashSet::new();
        if permissions
            .permissions
            .iter()
            .any(|code| !codes.insert(*code))
        {
            return Err(format!("{field} contains a duplicate permission code"));
        }
    }
    Ok(())
}

pub(in crate::sns::report) fn validate_sns_reward_event_evidence(
    event: &SnsRewardEvent,
) -> Result<(), String> {
    if event.end_timestamp_seconds.is_none() {
        return Err("reward event is missing canonical end_timestamp_seconds".to_string());
    }
    let mut proposal_ids = HashSet::new();
    if event
        .settled_proposals
        .iter()
        .any(|proposal| !proposal_ids.insert(proposal.id))
    {
        return Err("reward event contains duplicate settled proposal ids".to_string());
    }
    Ok(())
}

fn validate_principal(field: &str, value: &str) -> Result<(), SnsRewardCheckpointValidationError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_validation(format!("{field} {value} is invalid: {error}")))?;
    if principal.to_text() == value {
        Ok(())
    } else {
        Err(invalid_validation(format!(
            "{field} {value} is not canonical principal text"
        )))
    }
}

fn invalid_validation(reason: impl Into<String>) -> SnsRewardCheckpointValidationError {
    SnsRewardCheckpointValidationError {
        reason: reason.into(),
    }
}

///
/// SnsRewardCheckpointSummary
///
/// Pure recomputation result derived from raw checkpoint parameters and rows.
///

pub(in crate::sns::report) struct SnsRewardCheckpointSummary {
    pub(in crate::sns::report) aggregate_maturity_e8s_equivalent: u64,
    pub(in crate::sns::report) aggregate_staked_maturity_e8s_equivalent: u64,
    pub(in crate::sns::report) aggregate_combined_maturity_e8s_equivalent: u64,
    pub(in crate::sns::report) permission_entry_count: usize,
    pub(in crate::sns::report) unassessable_permission_code_count: usize,
    pub(in crate::sns::report) pending_maturity_disbursement_count: usize,
    pub(in crate::sns::report) auto_stake_maturity_enabled_count: usize,
    pub(in crate::sns::report) auto_stake_maturity_disabled_count: usize,
    pub(in crate::sns::report) auto_stake_maturity_unspecified_count: usize,
    pub(in crate::sns::report) manage_principals_grantable: Option<bool>,
    pub(in crate::sns::report) maturity_mint_conversion_observed_disabled:
        SnsPolicyObservationStatus,
    pub(in crate::sns::report) manual_maturity_staking_observed_disabled:
        SnsPolicyObservationStatus,
    pub(in crate::sns::report) maturity_conversion_policy_observed_status:
        SnsPolicyObservationStatus,
}

pub(in crate::sns::report) fn recompute_reward_checkpoint_summary(
    parameters: &SnsGovernanceParameters,
    rows: &[SnsRewardCheckpointRow],
) -> Result<SnsRewardCheckpointSummary, String> {
    let mut summary = SnsRewardCheckpointSummary {
        aggregate_maturity_e8s_equivalent: 0,
        aggregate_staked_maturity_e8s_equivalent: 0,
        aggregate_combined_maturity_e8s_equivalent: 0,
        permission_entry_count: 0,
        unassessable_permission_code_count: 0,
        pending_maturity_disbursement_count: 0,
        auto_stake_maturity_enabled_count: 0,
        auto_stake_maturity_disabled_count: 0,
        auto_stake_maturity_unspecified_count: 0,
        manage_principals_grantable: parameters
            .neuron_grantable_permissions
            .as_ref()
            .map(|permissions| permissions.permissions.contains(&2)),
        maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
        manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
        maturity_conversion_policy_observed_status: SnsPolicyObservationStatus::ObservedSatisfied,
    };
    let (parameter_mint, parameter_staking, parameter_unknown_count) =
        parameter_policy_observations(parameters);
    summary.maturity_mint_conversion_observed_disabled = parameter_mint;
    summary.manual_maturity_staking_observed_disabled = parameter_staking;
    summary.unassessable_permission_code_count = parameter_unknown_count;

    for row in rows {
        accumulate_reward_row(&mut summary, row)?;
    }
    summary.maturity_conversion_policy_observed_status = summary
        .maturity_mint_conversion_observed_disabled
        .combine(summary.manual_maturity_staking_observed_disabled);
    Ok(summary)
}

fn accumulate_reward_row(
    summary: &mut SnsRewardCheckpointSummary,
    row: &SnsRewardCheckpointRow,
) -> Result<(), String> {
    let combined = row.checked_combined_maturity().ok_or_else(|| {
        format!(
            "neuron {} combined maturity exceeds the native u64 contract",
            row.neuron_id
        )
    })?;
    if row.combined_maturity_e8s_equivalent != combined {
        return Err(format!(
            "neuron {} combined maturity does not match raw components",
            row.neuron_id
        ));
    }
    let (row_mint, row_staking) = row.derived_policy_observations();
    if row.maturity_mint_conversion_observed_disabled != row_mint
        || row.manual_maturity_staking_observed_disabled != row_staking
    {
        return Err(format!(
            "neuron {} policy observations do not match raw evidence",
            row.neuron_id
        ));
    }
    summary.aggregate_maturity_e8s_equivalent = checked_sum(
        summary.aggregate_maturity_e8s_equivalent,
        row.maturity_e8s_equivalent,
        "aggregate maturity",
    )?;
    summary.aggregate_staked_maturity_e8s_equivalent = checked_sum(
        summary.aggregate_staked_maturity_e8s_equivalent,
        row.staked_maturity_e8s_equivalent.unwrap_or(0),
        "aggregate staked maturity",
    )?;
    summary.aggregate_combined_maturity_e8s_equivalent = checked_sum(
        summary.aggregate_combined_maturity_e8s_equivalent,
        combined,
        "aggregate combined maturity",
    )?;
    accumulate_reward_row_counts(summary, row)?;
    summary.maturity_mint_conversion_observed_disabled = summary
        .maturity_mint_conversion_observed_disabled
        .combine(row_mint);
    summary.manual_maturity_staking_observed_disabled = summary
        .manual_maturity_staking_observed_disabled
        .combine(row_staking);
    Ok(())
}

fn accumulate_reward_row_counts(
    summary: &mut SnsRewardCheckpointSummary,
    row: &SnsRewardCheckpointRow,
) -> Result<(), String> {
    summary.permission_entry_count = summary
        .permission_entry_count
        .checked_add(row.permissions.len())
        .ok_or_else(|| "permission entry count overflow".to_string())?;
    summary.pending_maturity_disbursement_count = summary
        .pending_maturity_disbursement_count
        .checked_add(row.disburse_maturity_in_progress.len())
        .ok_or_else(|| "pending maturity disbursement count overflow".to_string())?;
    summary.unassessable_permission_code_count = summary
        .unassessable_permission_code_count
        .checked_add(
            row.permissions
                .iter()
                .flat_map(|permission| permission.permission_types.iter())
                .filter(|value| !(1..=10).contains(&value.code))
                .count(),
        )
        .ok_or_else(|| "unassessable permission code count overflow".to_string())?;
    match row.auto_stake_maturity {
        Some(true) => increment(&mut summary.auto_stake_maturity_enabled_count)?,
        Some(false) => increment(&mut summary.auto_stake_maturity_disabled_count)?,
        None => increment(&mut summary.auto_stake_maturity_unspecified_count)?,
    }
    Ok(())
}

fn parameter_policy_observations(
    parameters: &SnsGovernanceParameters,
) -> (
    SnsPolicyObservationStatus,
    SnsPolicyObservationStatus,
    usize,
) {
    let mut mint = SnsPolicyObservationStatus::ObservedSatisfied;
    let mut staking = SnsPolicyObservationStatus::ObservedSatisfied;
    let mut unknown_count = 0;
    for permissions in [
        parameters.neuron_claimer_permissions.as_ref(),
        parameters.neuron_grantable_permissions.as_ref(),
    ] {
        let Some(permissions) = permissions else {
            mint = mint.combine(SnsPolicyObservationStatus::Unassessable);
            staking = staking.combine(SnsPolicyObservationStatus::Unassessable);
            continue;
        };
        if permissions.permissions.is_empty() {
            mint = mint.combine(SnsPolicyObservationStatus::Unassessable);
            staking = staking.combine(SnsPolicyObservationStatus::Unassessable);
        }
        for code in &permissions.permissions {
            let (code_mint, code_staking) = permission_code_policy_observations(*code);
            mint = mint.combine(code_mint);
            staking = staking.combine(code_staking);
            if !(1..=10).contains(code) {
                unknown_count += 1;
            }
        }
    }
    (mint, staking, unknown_count)
}

fn checked_sum(left: u64, right: u64, field: &str) -> Result<u64, String> {
    left.checked_add(right)
        .ok_or_else(|| format!("{field} exceeds the native u64 contract"))
}

fn increment(value: &mut usize) -> Result<(), String> {
    *value = value
        .checked_add(1)
        .ok_or_else(|| "checkpoint count overflow".to_string())?;
    Ok(())
}
