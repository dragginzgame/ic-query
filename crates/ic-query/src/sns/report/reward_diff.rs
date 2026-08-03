//! Module: sns::report::reward_diff
//!
//! Responsibility: purely reconcile two untrusted SNS reward checkpoints.
//! Does not own: filesystem loading, live calls, checkpoint collection, or output writing.
//! Boundary: validates raw checkpoints, joins full neuron ids, and allocates only after exact reconciliation.

use crate::sns::report::{
    SNS_REWARD_DIFF_REPORT_SCHEMA_VERSION, SnsPolicyObservationStatus, SnsRewardAllocationStatus,
    SnsRewardCheckpointReport, SnsRewardCheckpointRow, SnsRewardCheckpointSummary,
    SnsRewardDiffCheckpointRef, SnsRewardDiffInvalidReason, SnsRewardDiffInvalidReasonKind,
    SnsRewardDiffReport, SnsRewardDiffRow, recompute_reward_checkpoint_summary,
    validate_sns_reward_checkpoint_report,
};

/// Build one local reward diff from two untrusted in-memory checkpoints.
#[must_use]
pub fn build_sns_reward_diff_report(
    before: &SnsRewardCheckpointReport,
    after: &SnsRewardCheckpointReport,
) -> SnsRewardDiffReport {
    let mut reasons = Vec::new();
    let before_valid = validate_checkpoint(
        before,
        SnsRewardDiffInvalidReasonKind::BeforeCheckpointInvalid,
        &mut reasons,
    );
    let after_valid = validate_checkpoint(
        after,
        SnsRewardDiffInvalidReasonKind::AfterCheckpointInvalid,
        &mut reasons,
    );
    validate_target_identity(before, after, &mut reasons);
    validate_checkpoint_order(before, after, &mut reasons);
    validate_reward_continuity(before, after, &mut reasons);

    let before_summary = recompute_summary(before);
    let after_summary = recompute_summary(after);
    validate_policy("before", before_summary.as_ref(), &mut reasons);
    validate_policy("after", after_summary.as_ref(), &mut reasons);

    let mut rows = if before_valid && after_valid {
        join_reward_rows(before, after, &mut reasons)
    } else {
        Vec::new()
    };
    let aggregate_before = before_summary
        .as_ref()
        .map(|summary| summary.aggregate_combined_maturity_e8s_equivalent);
    let aggregate_after = after_summary
        .as_ref()
        .map(|summary| summary.aggregate_combined_maturity_e8s_equivalent);
    let aggregate_delta = aggregate_delta(aggregate_before, aggregate_after);
    let summed_neuron_delta = sum_neuron_deltas(&rows, &mut reasons);
    let distributed = after.reward_event_after.distributed_e8s_equivalent;
    let distributed_signed = i128::from(distributed);
    let aggregate_reconciled = aggregate_delta == Some(distributed_signed);
    let per_neuron_reconciled = summed_neuron_delta == Some(distributed_signed);
    retain_reconciliation_failures(
        aggregate_delta,
        summed_neuron_delta,
        distributed,
        &mut reasons,
    );

    let allocation_status = if !reasons.is_empty() {
        SnsRewardAllocationStatus::Invalid
    } else if distributed == 0 {
        SnsRewardAllocationStatus::NoAllocation
    } else {
        populate_allocations(&mut rows, distributed);
        SnsRewardAllocationStatus::Valid
    };

    SnsRewardDiffReport {
        schema_version: SNS_REWARD_DIFF_REPORT_SCHEMA_VERSION,
        before: checkpoint_ref(before),
        after: checkpoint_ref(after),
        aggregate_before_combined_maturity_e8s_equivalent: aggregate_before,
        aggregate_after_combined_maturity_e8s_equivalent: aggregate_after,
        aggregate_maturity_delta_e8s_equivalent: aggregate_delta,
        summed_neuron_maturity_delta_e8s_equivalent: summed_neuron_delta,
        distributed_e8s_equivalent: distributed,
        aggregate_reconciled,
        per_neuron_reconciled,
        before_policy_status: before_summary
            .as_ref()
            .map(|summary| summary.maturity_conversion_policy_observed_status),
        after_policy_status: after_summary
            .as_ref()
            .map(|summary| summary.maturity_conversion_policy_observed_status),
        allocation_status,
        invalid_reasons: reasons,
        rows,
        checkpoint_content_authenticated: false,
    }
}

fn validate_checkpoint(
    report: &SnsRewardCheckpointReport,
    kind: SnsRewardDiffInvalidReasonKind,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) -> bool {
    match validate_sns_reward_checkpoint_report(report) {
        Ok(()) => true,
        Err(error) => {
            reasons.push(reason(kind, None, error.reason));
            false
        }
    }
}

fn recompute_summary(report: &SnsRewardCheckpointReport) -> Option<SnsRewardCheckpointSummary> {
    recompute_reward_checkpoint_summary(&report.parameters_before, &report.rows).ok()
}

fn validate_target_identity(
    before: &SnsRewardCheckpointReport,
    after: &SnsRewardCheckpointReport,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) {
    for (field, before_value, after_value) in [
        ("network", before.network.as_str(), after.network.as_str()),
        (
            "sns_wasm_canister_id",
            before.sns_wasm_canister_id.as_str(),
            after.sns_wasm_canister_id.as_str(),
        ),
        (
            "root_canister_id",
            before.root_canister_id.as_str(),
            after.root_canister_id.as_str(),
        ),
        (
            "governance_canister_id",
            before.governance_canister_id.as_str(),
            after.governance_canister_id.as_str(),
        ),
        (
            "ledger_canister_id",
            before.ledger_canister_id.as_str(),
            after.ledger_canister_id.as_str(),
        ),
        (
            "swap_canister_id",
            before.swap_canister_id.as_str(),
            after.swap_canister_id.as_str(),
        ),
        (
            "index_canister_id",
            before.index_canister_id.as_str(),
            after.index_canister_id.as_str(),
        ),
    ] {
        if before_value != after_value {
            reasons.push(reason(
                SnsRewardDiffInvalidReasonKind::TargetMismatch,
                None,
                format!("{field} differs: before {before_value}, after {after_value}"),
            ));
        }
    }
}

fn validate_checkpoint_order(
    before: &SnsRewardCheckpointReport,
    after: &SnsRewardCheckpointReport,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) {
    if after.collection_started_at_unix_secs < before.collection_completed_at_unix_secs {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::CheckpointOrder,
            None,
            format!(
                "after collection started at {}, before collection completed at {}",
                after.collection_started_at_unix_secs, before.collection_completed_at_unix_secs
            ),
        ));
    }
}

fn validate_reward_continuity(
    before: &SnsRewardCheckpointReport,
    after: &SnsRewardCheckpointReport,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) {
    match (
        before.reward_event_after.end_timestamp_seconds,
        after.reward_event_after.end_timestamp_seconds,
    ) {
        (Some(before_end), Some(after_end)) if after_end > before_end => {}
        (before_end, after_end) => reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::RewardEventOrder,
            None,
            format!(
                "reward event end timestamps are not strictly increasing: before {before_end:?}, after {after_end:?}"
            ),
        )),
    }

    if after.reward_event_after.actual_timestamp_seconds <= before.collection_completed_at_unix_secs
    {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::RewardEventCoverage,
            None,
            format!(
                "after reward distribution ran at {}, not after before collection completed at {}",
                after.reward_event_after.actual_timestamp_seconds,
                before.collection_completed_at_unix_secs
            ),
        ));
    }

    let rounds = after.reward_event_after.rounds_since_last_distribution;
    let expected_round = rounds
        .filter(|rounds| *rounds > 0)
        .and_then(|rounds| before.reward_event_after.round.checked_add(rounds));
    if expected_round != Some(after.reward_event_after.round) {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::RewardEventContinuity,
            None,
            format!(
                "before round {}, after round {}, after rounds_since_last_distribution {rounds:?}",
                before.reward_event_after.round, after.reward_event_after.round
            ),
        ));
    }
}

fn validate_policy(
    checkpoint: &str,
    summary: Option<&SnsRewardCheckpointSummary>,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) {
    let status = summary.map(|summary| summary.maturity_conversion_policy_observed_status);
    if status != Some(SnsPolicyObservationStatus::ObservedSatisfied) {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::PolicyNotObservedSatisfied,
            None,
            format!("{checkpoint} recomputed policy status is {status:?}"),
        ));
    }
}

fn join_reward_rows(
    before: &SnsRewardCheckpointReport,
    after: &SnsRewardCheckpointReport,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) -> Vec<SnsRewardDiffRow> {
    let mut rows = Vec::with_capacity(before.rows.len().max(after.rows.len()));
    let mut before_rows = before.rows.iter().peekable();
    let mut after_rows = after.rows.iter().peekable();
    while before_rows.peek().is_some() || after_rows.peek().is_some() {
        match (before_rows.peek(), after_rows.peek()) {
            (Some(before_row), Some(after_row)) if before_row.neuron_id == after_row.neuron_id => {
                rows.push(join_matched_row(before_row, after_row, reasons));
                before_rows.next();
                after_rows.next();
            }
            (Some(before_row), Some(after_row)) if before_row.neuron_id < after_row.neuron_id => {
                rows.push(join_missing_after_row(before_row, reasons));
                before_rows.next();
            }
            (Some(_) | None, Some(after_row)) => {
                rows.push(join_new_row(before, after, after_row, reasons));
                after_rows.next();
            }
            (Some(before_row), None) => {
                rows.push(join_missing_after_row(before_row, reasons));
                before_rows.next();
            }
            (None, None) => break,
        }
    }
    rows
}

fn join_matched_row(
    before: &SnsRewardCheckpointRow,
    after: &SnsRewardCheckpointRow,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) -> SnsRewardDiffRow {
    if before.created_timestamp_seconds != after.created_timestamp_seconds {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::NeuronCreationTimestampChanged,
            Some(&before.neuron_id),
            format!(
                "creation timestamp changed from {} to {}",
                before.created_timestamp_seconds, after.created_timestamp_seconds
            ),
        ));
    }
    let delta = i128::from(after.combined_maturity_e8s_equivalent)
        - i128::from(before.combined_maturity_e8s_equivalent);
    if delta < 0 {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::NegativeMaturityDelta,
            Some(&before.neuron_id),
            format!(
                "combined maturity changed from {} to {}",
                before.combined_maturity_e8s_equivalent, after.combined_maturity_e8s_equivalent
            ),
        ));
    }
    row_from_pair(Some(before), Some(after), delta, false, false)
}

fn join_missing_after_row(
    before: &SnsRewardCheckpointRow,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) -> SnsRewardDiffRow {
    reasons.push(reason(
        SnsRewardDiffInvalidReasonKind::NeuronMissingAfter,
        Some(&before.neuron_id),
        "neuron from the earlier checkpoint is absent later",
    ));
    row_from_pair(
        Some(before),
        None,
        -i128::from(before.combined_maturity_e8s_equivalent),
        false,
        true,
    )
}

fn join_new_row(
    before_report: &SnsRewardCheckpointReport,
    after_report: &SnsRewardCheckpointReport,
    after: &SnsRewardCheckpointRow,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) -> SnsRewardDiffRow {
    let latest_supported_creation = after_report
        .collection_completed_at_unix_secs
        .min(after_report.reward_event_after.actual_timestamp_seconds);
    if after.created_timestamp_seconds <= before_report.collection_completed_at_unix_secs
        || after.created_timestamp_seconds > latest_supported_creation
    {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::NewNeuronCreationUnexplained,
            Some(&after.neuron_id),
            format!(
                "creation timestamp {} is not in ({}, {}]",
                after.created_timestamp_seconds,
                before_report.collection_completed_at_unix_secs,
                latest_supported_creation
            ),
        ));
    }
    row_from_pair(
        None,
        Some(after),
        i128::from(after.combined_maturity_e8s_equivalent),
        true,
        false,
    )
}

fn row_from_pair(
    before: Option<&SnsRewardCheckpointRow>,
    after: Option<&SnsRewardCheckpointRow>,
    delta: i128,
    new_neuron: bool,
    missing_after: bool,
) -> SnsRewardDiffRow {
    let neuron_id = before
        .or(after)
        .map_or_else(String::new, |row| row.neuron_id.clone());
    SnsRewardDiffRow {
        neuron_id,
        before_combined_maturity_e8s_equivalent: before
            .map(|row| row.combined_maturity_e8s_equivalent),
        after_combined_maturity_e8s_equivalent: after
            .map(|row| row.combined_maturity_e8s_equivalent),
        maturity_delta_e8s_equivalent: delta,
        new_neuron,
        missing_after,
        before_created_timestamp_seconds: before.map(|row| row.created_timestamp_seconds),
        after_created_timestamp_seconds: after.map(|row| row.created_timestamp_seconds),
        before_maturity_mint_conversion_observed_disabled: before
            .map(|row| row.maturity_mint_conversion_observed_disabled),
        after_maturity_mint_conversion_observed_disabled: after
            .map(|row| row.maturity_mint_conversion_observed_disabled),
        before_manual_maturity_staking_observed_disabled: before
            .map(|row| row.manual_maturity_staking_observed_disabled),
        after_manual_maturity_staking_observed_disabled: after
            .map(|row| row.manual_maturity_staking_observed_disabled),
        before_pending_maturity_disbursement_count: before
            .map(|row| row.disburse_maturity_in_progress.len()),
        after_pending_maturity_disbursement_count: after
            .map(|row| row.disburse_maturity_in_progress.len()),
        policy_evidence_changed: policy_evidence_changed(before, after),
        allocation_numerator_e8s_equivalent: None,
        allocation_denominator_e8s_equivalent: None,
    }
}

fn policy_evidence_changed(
    before: Option<&SnsRewardCheckpointRow>,
    after: Option<&SnsRewardCheckpointRow>,
) -> bool {
    match (before, after) {
        (Some(before), Some(after)) => {
            before.permissions != after.permissions
                || before.disburse_maturity_in_progress != after.disburse_maturity_in_progress
                || before.auto_stake_maturity != after.auto_stake_maturity
        }
        _ => true,
    }
}

fn aggregate_delta(before: Option<u64>, after: Option<u64>) -> Option<i128> {
    Some(i128::from(after?) - i128::from(before?))
}

fn sum_neuron_deltas(
    rows: &[SnsRewardDiffRow],
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) -> Option<i128> {
    let mut total = 0_i128;
    for row in rows {
        let Some(next) = total.checked_add(row.maturity_delta_e8s_equivalent) else {
            reasons.push(reason(
                SnsRewardDiffInvalidReasonKind::Arithmetic,
                None,
                "sum of per-neuron maturity deltas exceeds i128",
            ));
            return None;
        };
        total = next;
    }
    Some(total)
}

fn retain_reconciliation_failures(
    aggregate_delta: Option<i128>,
    summed_neuron_delta: Option<i128>,
    distributed: u64,
    reasons: &mut Vec<SnsRewardDiffInvalidReason>,
) {
    let distributed = i128::from(distributed);
    if aggregate_delta != Some(distributed) {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::AggregateReconciliation,
            None,
            format!("aggregate maturity delta {aggregate_delta:?} does not equal distributed {distributed}"),
        ));
    }
    if summed_neuron_delta != Some(distributed) {
        reasons.push(reason(
            SnsRewardDiffInvalidReasonKind::PerNeuronReconciliation,
            None,
            format!("summed neuron maturity delta {summed_neuron_delta:?} does not equal distributed {distributed}"),
        ));
    }
}

fn populate_allocations(rows: &mut [SnsRewardDiffRow], denominator: u64) {
    for row in rows {
        row.allocation_numerator_e8s_equivalent =
            u64::try_from(row.maturity_delta_e8s_equivalent).ok();
        row.allocation_denominator_e8s_equivalent = Some(denominator);
    }
}

fn checkpoint_ref(report: &SnsRewardCheckpointReport) -> SnsRewardDiffCheckpointRef {
    SnsRewardDiffCheckpointRef {
        network: report.network.clone(),
        sns_wasm_canister_id: report.sns_wasm_canister_id.clone(),
        id: report.id,
        name: report.name.clone(),
        root_canister_id: report.root_canister_id.clone(),
        governance_canister_id: report.governance_canister_id.clone(),
        ledger_canister_id: report.ledger_canister_id.clone(),
        swap_canister_id: report.swap_canister_id.clone(),
        index_canister_id: report.index_canister_id.clone(),
        source_endpoint: report.source_endpoint.clone(),
        collection_completed_at_unix_secs: report.collection_completed_at_unix_secs,
        reward_event_end_timestamp_seconds: report.reward_event_after.end_timestamp_seconds,
        reward_event_actual_timestamp_seconds: report.reward_event_after.actual_timestamp_seconds,
        reward_event_round: report.reward_event_after.round,
        rounds_since_last_distribution: report.reward_event_after.rounds_since_last_distribution,
        distributed_e8s_equivalent: report.reward_event_after.distributed_e8s_equivalent,
    }
}

fn reason(
    kind: SnsRewardDiffInvalidReasonKind,
    neuron_id: Option<&str>,
    detail: impl Into<String>,
) -> SnsRewardDiffInvalidReason {
    SnsRewardDiffInvalidReason {
        kind,
        neuron_id: neuron_id.map(str::to_string),
        detail: detail.into(),
    }
}
