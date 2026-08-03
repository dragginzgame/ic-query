//! Module: nns::governance::text::metrics
//!
//! Responsibility: render native NNS Governance cached-metrics reports.
//! Does not own: economics, reward events, maturity modulation, or process output.
//! Boundary: formats scalar, bucket, and neuron-subset metrics without changing data.

use super::{
    super::{
        NnsGovernanceMetricBucket, NnsGovernanceMetricsReport, NnsGovernanceNeuronSubsetMetrics,
    },
    context_lines,
};
use crate::text_value::optional_u64_text;
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
