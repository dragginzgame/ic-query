//! Module: nns::governance::text
//!
//! Responsibility: render direct NNS Governance reports as human-readable text.
//! Does not own: live calls, report construction, caching, or process output.
//! Boundary: formats native values without changing JSON or report data.

use super::{
    NnsGovernanceDecimal, NnsGovernanceEconomicsReport, NnsGovernanceMaturityModulationReport,
    NnsGovernanceMetricBucket, NnsGovernanceMetricsReport, NnsGovernanceNeuronSubsetMetrics,
    NnsGovernancePercentage, NnsGovernanceReportContext, NnsGovernanceRewardEventReport,
};
use crate::text_value::{optional_u64_text, sanitize_text};
use std::fmt::Display;

macro_rules! push_scalar_metrics {
    ($lines:expr, $metrics:expr, $($field:ident),+ $(,)?) => {
        $(
            $lines.push(format!("{}: {}", stringify!($field), $metrics.$field));
        )+
    };
}

macro_rules! push_metric_buckets {
    ($lines:expr, $metrics:expr, $($field:ident),+ $(,)?) => {
        $(
            $lines.push(format!(
                "{}: {}",
                stringify!($field),
                metric_buckets_text(&$metrics.$field)
            ));
        )+
    };
}

/// Render one NNS Governance economics report.
#[must_use]
pub fn nns_governance_economics_report_text(report: &NnsGovernanceEconomicsReport) -> String {
    let economics = &report.economics;
    let mut lines = context_lines(&report.context);
    lines.extend([
        format!(
            "neuron_minimum_stake_e8s: {}",
            economics.neuron_minimum_stake_e8s
        ),
        format!(
            "max_proposals_to_keep_per_topic: {}",
            economics.max_proposals_to_keep_per_topic
        ),
        format!(
            "neuron_management_fee_per_proposal_e8s: {}",
            economics.neuron_management_fee_per_proposal_e8s
        ),
        format!("reject_cost_e8s: {}", economics.reject_cost_e8s),
        format!("transaction_fee_e8s: {}", economics.transaction_fee_e8s),
        format!(
            "neuron_spawn_dissolve_delay_seconds: {}",
            economics.neuron_spawn_dissolve_delay_seconds
        ),
        format!("minimum_icp_xdr_rate: {}", economics.minimum_icp_xdr_rate),
        format!(
            "maximum_node_provider_rewards_e8s: {}",
            economics.maximum_node_provider_rewards_e8s
        ),
    ]);
    if let Some(fund) = economics.neurons_fund_economics.as_ref() {
        lines.extend([
            format!(
                "neurons_fund_economics.maximum_icp_xdr_rate: {}",
                percentage_text(fund.maximum_icp_xdr_rate.as_ref())
            ),
            format!(
                "neurons_fund_economics.max_theoretical_neurons_fund_participation_amount_xdr: {}",
                decimal_text(
                    fund.max_theoretical_neurons_fund_participation_amount_xdr
                        .as_ref()
                )
            ),
            format!(
                "neurons_fund_economics.minimum_icp_xdr_rate: {}",
                percentage_text(fund.minimum_icp_xdr_rate.as_ref())
            ),
        ]);
        if let Some(curve) = fund
            .neurons_fund_matched_funding_curve_coefficients
            .as_ref()
        {
            lines.extend([
                format!(
                    "neurons_fund_economics.neurons_fund_matched_funding_curve_coefficients.contribution_threshold_xdr: {}",
                    decimal_text(curve.contribution_threshold_xdr.as_ref())
                ),
                format!(
                    "neurons_fund_economics.neurons_fund_matched_funding_curve_coefficients.one_third_participation_milestone_xdr: {}",
                    decimal_text(curve.one_third_participation_milestone_xdr.as_ref())
                ),
                format!(
                    "neurons_fund_economics.neurons_fund_matched_funding_curve_coefficients.full_participation_milestone_xdr: {}",
                    decimal_text(curve.full_participation_milestone_xdr.as_ref())
                ),
            ]);
        } else {
            lines.push(
                "neurons_fund_economics.neurons_fund_matched_funding_curve_coefficients: -"
                    .to_string(),
            );
        }
    } else {
        lines.push("neurons_fund_economics: -".to_string());
    }
    if let Some(voting) = economics.voting_power_economics.as_ref() {
        lines.extend([
            format!(
                "voting_power_economics.start_reducing_voting_power_after_seconds: {}",
                optional_u64_text(voting.start_reducing_voting_power_after_seconds)
            ),
            format!(
                "voting_power_economics.clear_following_after_seconds: {}",
                optional_u64_text(voting.clear_following_after_seconds)
            ),
            format!(
                "voting_power_economics.neuron_minimum_dissolve_delay_to_vote_seconds: {}",
                optional_u64_text(voting.neuron_minimum_dissolve_delay_to_vote_seconds)
            ),
        ]);
    } else {
        lines.push("voting_power_economics: -".to_string());
    }
    lines.join("\n")
}

/// Render one NNS Governance metrics report.
#[must_use]
pub fn nns_governance_metrics_report_text(report: &NnsGovernanceMetricsReport) -> String {
    let metrics = &report.metrics;
    let mut lines = context_lines(&report.context);
    push_scalar_metrics!(
        lines,
        metrics,
        total_maturity_e8s_equivalent,
        dissolving_neurons_staked_maturity_e8s_equivalent_sum,
        garbage_collectable_neurons_count,
        neurons_with_invalid_stake_count,
        ect_neuron_count,
        total_supply_icp,
        neurons_with_less_than_6_months_dissolve_delay_count,
        dissolved_neurons_count,
        community_fund_total_maturity_e8s_equivalent,
        total_staked_e8s_seed,
        total_staked_maturity_e8s_equivalent_ect,
        total_staked_e8s,
        not_dissolving_neurons_count,
        total_locked_e8s,
        neurons_fund_total_active_neurons,
        total_staked_maturity_e8s_equivalent,
        total_staked_e8s_ect,
        not_dissolving_neurons_staked_maturity_e8s_equivalent_sum,
        dissolved_neurons_e8s,
        neurons_with_less_than_6_months_dissolve_delay_e8s,
        dissolving_neurons_count,
        total_staked_maturity_e8s_equivalent_seed,
        community_fund_total_staked_e8s,
        timestamp_seconds,
        seed_neuron_count,
        spawning_neurons_count,
        total_maturity_disbursements_in_progress_e8s_equivalent
    );
    lines.extend([
        format!(
            "total_voting_power_non_self_authenticating_controller: {}",
            optional_u64_text(metrics.total_voting_power_non_self_authenticating_controller)
        ),
        format!(
            "total_staked_e8s_non_self_authenticating_controller: {}",
            optional_u64_text(metrics.total_staked_e8s_non_self_authenticating_controller)
        ),
    ]);
    push_metric_buckets!(
        lines,
        metrics,
        not_dissolving_neurons_e8s_buckets,
        dissolving_neurons_staked_maturity_e8s_equivalent_buckets,
        not_dissolving_neurons_count_buckets,
        not_dissolving_neurons_e8s_buckets_ect,
        dissolving_neurons_e8s_buckets_seed,
        not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets,
        dissolving_neurons_count_buckets,
        dissolving_neurons_e8s_buckets_ect,
        dissolving_neurons_e8s_buckets,
        not_dissolving_neurons_e8s_buckets_seed
    );
    push_subset_metrics(
        &mut lines,
        "non_self_authenticating_controller_neuron_subset_metrics",
        metrics
            .non_self_authenticating_controller_neuron_subset_metrics
            .as_ref(),
    );
    push_subset_metrics(
        &mut lines,
        "public_neuron_subset_metrics",
        metrics.public_neuron_subset_metrics.as_ref(),
    );
    push_subset_metrics(
        &mut lines,
        "declining_voting_power_neuron_subset_metrics",
        metrics
            .declining_voting_power_neuron_subset_metrics
            .as_ref(),
    );
    push_subset_metrics(
        &mut lines,
        "fully_lost_voting_power_neuron_subset_metrics",
        metrics
            .fully_lost_voting_power_neuron_subset_metrics
            .as_ref(),
    );
    lines.join("\n")
}

/// Render one latest NNS Governance reward-event report.
#[must_use]
pub fn nns_governance_reward_event_report_text(report: &NnsGovernanceRewardEventReport) -> String {
    let event = &report.reward_event;
    let mut lines = context_lines(&report.context);
    lines.extend([
        format!(
            "rounds_since_last_distribution: {}",
            optional_u64_text(event.rounds_since_last_distribution)
        ),
        format!("day_after_genesis: {}", event.day_after_genesis),
        format!(
            "actual_timestamp_seconds: {}",
            event.actual_timestamp_seconds
        ),
        format!(
            "total_available_e8s_equivalent: {}",
            event.total_available_e8s_equivalent
        ),
        format!(
            "latest_round_available_e8s_equivalent: {}",
            optional_u64_text(event.latest_round_available_e8s_equivalent)
        ),
        format!(
            "distributed_e8s_equivalent: {}",
            event.distributed_e8s_equivalent
        ),
        format!(
            "settled_proposals: {}",
            if event.settled_proposals.is_empty() {
                "-".to_string()
            } else {
                event
                    .settled_proposals
                    .iter()
                    .map(|proposal| proposal.id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ),
    ]);
    lines.join("\n")
}

/// Render one NNS Governance maturity-modulation report.
#[must_use]
pub fn nns_governance_maturity_modulation_report_text(
    report: &NnsGovernanceMaturityModulationReport,
) -> String {
    let mut lines = context_lines(&report.context);
    if let Some(modulation) = report.maturity_modulation.as_ref() {
        lines.extend([
            format!(
                "current_value_permyriad: {}",
                modulation
                    .current_value_permyriad
                    .map_or_else(|| "-".to_string(), |value| value.to_string())
            ),
            format!(
                "updated_at_timestamp_seconds: {}",
                optional_u64_text(modulation.updated_at_timestamp_seconds)
            ),
        ]);
    } else {
        lines.push("maturity_modulation: -".to_string());
    }
    lines.join("\n")
}

fn context_lines(context: &NnsGovernanceReportContext) -> Vec<String> {
    vec![
        format!("network: {}", sanitize_text(&context.network)),
        format!("governance_canister_id: {}", context.governance_canister_id),
        format!("fetched_at: {}", sanitize_text(&context.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&context.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&context.fetched_by)),
    ]
}

fn percentage_text(value: Option<&NnsGovernancePercentage>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |value| optional_u64_text(value.basis_points),
    )
}

fn decimal_text(value: Option<&NnsGovernanceDecimal>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |value| {
            value
                .human_readable
                .as_deref()
                .map_or_else(|| "-".to_string(), sanitize_text)
        },
    )
}

fn metric_buckets_text<Value>(buckets: &[NnsGovernanceMetricBucket<Value>]) -> String
where
    Value: Display,
{
    if buckets.is_empty() {
        return "-".to_string();
    }
    buckets
        .iter()
        .map(|bucket| format!("{}={}", bucket.key, bucket.value))
        .collect::<Vec<_>>()
        .join(", ")
}

fn push_subset_metrics(
    lines: &mut Vec<String>,
    prefix: &str,
    subset: Option<&NnsGovernanceNeuronSubsetMetrics>,
) {
    let Some(subset) = subset else {
        lines.push(format!("{prefix}: -"));
        return;
    };
    for (name, value) in [
        ("count", subset.count),
        ("total_staked_e8s", subset.total_staked_e8s),
        (
            "total_maturity_e8s_equivalent",
            subset.total_maturity_e8s_equivalent,
        ),
        (
            "total_staked_maturity_e8s_equivalent",
            subset.total_staked_maturity_e8s_equivalent,
        ),
        ("total_voting_power", subset.total_voting_power),
        (
            "total_deciding_voting_power",
            subset.total_deciding_voting_power,
        ),
        (
            "total_potential_voting_power",
            subset.total_potential_voting_power,
        ),
    ] {
        lines.push(format!("{prefix}.{name}: {}", optional_u64_text(value)));
    }
    for (name, buckets) in [
        ("count_buckets", &subset.count_buckets),
        ("staked_e8s_buckets", &subset.staked_e8s_buckets),
        (
            "maturity_e8s_equivalent_buckets",
            &subset.maturity_e8s_equivalent_buckets,
        ),
        (
            "staked_maturity_e8s_equivalent_buckets",
            &subset.staked_maturity_e8s_equivalent_buckets,
        ),
        ("voting_power_buckets", &subset.voting_power_buckets),
        (
            "deciding_voting_power_buckets",
            &subset.deciding_voting_power_buckets,
        ),
        (
            "potential_voting_power_buckets",
            &subset.potential_voting_power_buckets,
        ),
    ] {
        lines.push(format!("{prefix}.{name}: {}", metric_buckets_text(buckets)));
    }
}
