//! Module: sns::report::model::reports::neurons::diff
//!
//! Responsibility: local SNS reward-checkpoint reconciliation DTOs.
//! Does not own: checkpoint collection, filesystem loading, or live source calls.
//! Boundary: preserves joined raw maturity deltas, typed invalid reasons, and allocations.

use super::SnsPolicyObservationStatus;
use serde::{Deserialize, Serialize};

///
/// SnsRewardAllocationStatus
///
/// Reconciliation outcome for one pair of SNS reward checkpoints.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsRewardAllocationStatus {
    /// Every invariant reconciled to one positive native reward distribution.
    Valid,
    /// Every invariant reconciled to a native distribution of zero.
    NoAllocation,
    /// At least one checkpoint, continuity, policy, or reconciliation invariant failed.
    Invalid,
}

impl SnsRewardAllocationStatus {
    /// Return the stable report label for this allocation status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::NoAllocation => "no_allocation",
            Self::Invalid => "invalid",
        }
    }
}

///
/// SnsRewardDiffInvalidReasonKind
///
/// Stable category for one failed reward-diff invariant.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsRewardDiffInvalidReasonKind {
    /// The earlier checkpoint failed pure raw-evidence validation.
    BeforeCheckpointInvalid,
    /// The later checkpoint failed pure raw-evidence validation.
    AfterCheckpointInvalid,
    /// A stable network or canister identity differs between checkpoints.
    TargetMismatch,
    /// The later collection started before the earlier collection completed.
    CheckpointOrder,
    /// A recomputed checkpoint policy is not observed satisfied.
    PolicyNotObservedSatisfied,
    /// Canonical reward-event timestamps are missing or not strictly increasing.
    RewardEventOrder,
    /// The later distribution did not occur after the earlier collection completed.
    RewardEventCoverage,
    /// Native reward round continuity does not describe the immediate next event.
    RewardEventContinuity,
    /// A neuron present before is absent after.
    NeuronMissingAfter,
    /// A later-only neuron cannot truthfully receive a synthetic zero before value.
    NewNeuronCreationUnexplained,
    /// A matched neuron changed its reported creation timestamp.
    NeuronCreationTimestampChanged,
    /// A neuron's combined maturity decreased.
    NegativeMaturityDelta,
    /// Aggregate before/after maturity does not reconcile to the native distribution.
    AggregateReconciliation,
    /// The sum of joined neuron deltas does not reconcile to the native distribution.
    PerNeuronReconciliation,
    /// Checked arithmetic could not represent a required value.
    Arithmetic,
}

impl SnsRewardDiffInvalidReasonKind {
    /// Return the stable report label for this invalid-reason category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeforeCheckpointInvalid => "before_checkpoint_invalid",
            Self::AfterCheckpointInvalid => "after_checkpoint_invalid",
            Self::TargetMismatch => "target_mismatch",
            Self::CheckpointOrder => "checkpoint_order",
            Self::PolicyNotObservedSatisfied => "policy_not_observed_satisfied",
            Self::RewardEventOrder => "reward_event_order",
            Self::RewardEventCoverage => "reward_event_coverage",
            Self::RewardEventContinuity => "reward_event_continuity",
            Self::NeuronMissingAfter => "neuron_missing_after",
            Self::NewNeuronCreationUnexplained => "new_neuron_creation_unexplained",
            Self::NeuronCreationTimestampChanged => "neuron_creation_timestamp_changed",
            Self::NegativeMaturityDelta => "negative_maturity_delta",
            Self::AggregateReconciliation => "aggregate_reconciliation",
            Self::PerNeuronReconciliation => "per_neuron_reconciliation",
            Self::Arithmetic => "arithmetic",
        }
    }
}

///
/// SnsRewardDiffInvalidReason
///
/// One typed failed invariant retained in an invalid reward diff.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsRewardDiffInvalidReason {
    /// Stable machine-readable reason category.
    pub kind: SnsRewardDiffInvalidReasonKind,
    /// Full neuron identifier when the failure belongs to one joined row.
    pub neuron_id: Option<String>,
    /// Deterministic human-readable detail retaining compared raw values.
    pub detail: String,
}

///
/// SnsRewardDiffCheckpointRef
///
/// Stable identity and event position retained for one compared checkpoint.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsRewardDiffCheckpointRef {
    /// Requested network identity.
    pub network: String,
    /// Stable SNS-W canister principal.
    pub sns_wasm_canister_id: String,
    /// Mutable SNS-W list position retained only as display metadata.
    pub id: usize,
    /// Mutable SNS name retained only as display metadata.
    pub name: String,
    /// Stable SNS Root canister principal.
    pub root_canister_id: String,
    /// Stable SNS Governance canister principal.
    pub governance_canister_id: String,
    /// Stable SNS ledger canister principal.
    pub ledger_canister_id: String,
    /// Stable SNS swap canister principal.
    pub swap_canister_id: String,
    /// Stable SNS index canister principal.
    pub index_canister_id: String,
    /// Explicit source endpoint retained as provenance, not target identity.
    pub source_endpoint: String,
    /// Collection completion timestamp from the checkpoint.
    pub collection_completed_at_unix_secs: u64,
    /// Canonical reward-event position.
    pub reward_event_end_timestamp_seconds: Option<u64>,
    /// Native timestamp at which the represented reward distribution actually ran.
    pub reward_event_actual_timestamp_seconds: u64,
    /// Deprecated native round retained as continuity evidence.
    pub reward_event_round: u64,
    /// Native number of rounds covered by the event when supplied.
    pub rounds_since_last_distribution: Option<u64>,
    /// Exact maturity distributed by this checkpoint's native reward event.
    pub distributed_e8s_equivalent: u64,
}

///
/// SnsRewardDiffRow
///
/// Joined raw maturity and policy evidence for one full SNS neuron identifier.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsRewardDiffRow {
    /// Exact 32-byte neuron identifier as lowercase hexadecimal text.
    pub neuron_id: String,
    /// Earlier combined maturity, or no value for a supported new neuron.
    pub before_combined_maturity_e8s_equivalent: Option<u64>,
    /// Later combined maturity, or no value when the neuron disappeared.
    pub after_combined_maturity_e8s_equivalent: Option<u64>,
    /// Raw signed later-minus-earlier combined maturity delta.
    pub maturity_delta_e8s_equivalent: i128,
    /// Whether the neuron was first observed after the earlier checkpoint.
    pub new_neuron: bool,
    /// Whether an earlier neuron was absent from the later checkpoint.
    pub missing_after: bool,
    /// Earlier creation timestamp when the neuron existed.
    pub before_created_timestamp_seconds: Option<u64>,
    /// Later creation timestamp when the neuron existed.
    pub after_created_timestamp_seconds: Option<u64>,
    /// Earlier neuron-local mint-conversion observation when present.
    pub before_maturity_mint_conversion_observed_disabled: Option<SnsPolicyObservationStatus>,
    /// Later neuron-local mint-conversion observation when present.
    pub after_maturity_mint_conversion_observed_disabled: Option<SnsPolicyObservationStatus>,
    /// Earlier neuron-local manual-staking observation when present.
    pub before_manual_maturity_staking_observed_disabled: Option<SnsPolicyObservationStatus>,
    /// Later neuron-local manual-staking observation when present.
    pub after_manual_maturity_staking_observed_disabled: Option<SnsPolicyObservationStatus>,
    /// Earlier pending maturity-disbursement count.
    pub before_pending_maturity_disbursement_count: Option<usize>,
    /// Later pending maturity-disbursement count.
    pub after_pending_maturity_disbursement_count: Option<usize>,
    /// Whether raw permission, pending-disbursement, or auto-stake evidence changed.
    pub policy_evidence_changed: bool,
    /// Allocation numerator, populated only for a valid positive distribution.
    pub allocation_numerator_e8s_equivalent: Option<u64>,
    /// Shared allocation denominator, populated only for a valid positive distribution.
    pub allocation_denominator_e8s_equivalent: Option<u64>,
}

///
/// SnsRewardDiffReport
///
/// Pure local reconciliation of two untrusted SNS reward checkpoints.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsRewardDiffReport {
    /// Reward-diff report schema version.
    pub schema_version: u32,
    /// Earlier checkpoint identity and event position.
    pub before: SnsRewardDiffCheckpointRef,
    /// Later checkpoint identity and event position.
    pub after: SnsRewardDiffCheckpointRef,
    /// Recomputed earlier aggregate combined maturity when representable.
    pub aggregate_before_combined_maturity_e8s_equivalent: Option<u64>,
    /// Recomputed later aggregate combined maturity when representable.
    pub aggregate_after_combined_maturity_e8s_equivalent: Option<u64>,
    /// Raw signed later-minus-earlier aggregate maturity delta when representable.
    pub aggregate_maturity_delta_e8s_equivalent: Option<i128>,
    /// Checked sum of every joined raw signed neuron delta when representable.
    pub summed_neuron_maturity_delta_e8s_equivalent: Option<i128>,
    /// Native distributed value against which both delta paths reconcile.
    pub distributed_e8s_equivalent: u64,
    /// Whether aggregate before/after maturity exactly matches the native distribution.
    pub aggregate_reconciled: bool,
    /// Whether the sum of joined neuron deltas exactly matches the native distribution.
    pub per_neuron_reconciled: bool,
    /// Recomputed earlier global maturity-conversion policy status when available.
    pub before_policy_status: Option<SnsPolicyObservationStatus>,
    /// Recomputed later global maturity-conversion policy status when available.
    pub after_policy_status: Option<SnsPolicyObservationStatus>,
    /// Typed allocation outcome.
    pub allocation_status: SnsRewardAllocationStatus,
    /// Every failed comparison and reconciliation invariant.
    pub invalid_reasons: Vec<SnsRewardDiffInvalidReason>,
    /// Canonically neuron-id-ordered joined rows.
    pub rows: Vec<SnsRewardDiffRow>,
    /// Always false because local JSON evidence has no content-authenticity proof.
    pub checkpoint_content_authenticated: bool,
}
