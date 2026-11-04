//! Module: nns::governance::economics
//!
//! Responsibility: render native NNS Governance economics reports.
//! Does not own: metrics, reward events, maturity modulation, or process output.
//! Boundary: formats economics values and their native optional wrappers without changing data.

use crate::nns::governance::context_lines;
use crate::text_value::field;
use crate::text_value::optional_u64_text;
use ic_host_text::escape::terminal_text;
use ic_query::nns::governance::NnsGovernanceDecimal;
use ic_query::nns::governance::NnsGovernanceEconomicsReport;
use ic_query::nns::governance::NnsGovernancePercentage;

/// Render one NNS Governance economics report.
#[must_use]
pub fn nns_governance_economics_report_text(report: &NnsGovernanceEconomicsReport) -> String {
    let economics = &report.economics;
    let mut lines = context_lines(&report.context);
    lines.extend([
        field(
            "neuron_minimum_stake_e8s",
            &(economics.neuron_minimum_stake_e8s),
        ),
        field(
            "max_proposals_to_keep_per_topic",
            &(economics.max_proposals_to_keep_per_topic),
        ),
        field(
            "neuron_management_fee_per_proposal_e8s",
            &(economics.neuron_management_fee_per_proposal_e8s),
        ),
        field("reject_cost_e8s", &(economics.reject_cost_e8s)),
        field("transaction_fee_e8s", &(economics.transaction_fee_e8s)),
        field(
            "neuron_spawn_dissolve_delay_seconds",
            &(economics.neuron_spawn_dissolve_delay_seconds),
        ),
        field("minimum_icp_xdr_rate", &(economics.minimum_icp_xdr_rate)),
        field(
            "maximum_node_provider_rewards_e8s",
            &(economics.maximum_node_provider_rewards_e8s),
        ),
    ]);
    if let Some(fund) = economics.neurons_fund_economics.as_ref() {
        lines.extend([
            field(
                "neurons_fund_economics.maximum_icp_xdr_rate",
                &(percentage_text(fund.maximum_icp_xdr_rate.as_ref())),
            ),
            field(
                "neurons_fund_economics.max_theoretical_neurons_fund_participation_amount_xdr",
                &(decimal_text(
                    fund.max_theoretical_neurons_fund_participation_amount_xdr
                        .as_ref(),
                )),
            ),
            field(
                "neurons_fund_economics.minimum_icp_xdr_rate",
                &(percentage_text(fund.minimum_icp_xdr_rate.as_ref())),
            ),
        ]);
        if let Some(curve) = fund
            .neurons_fund_matched_funding_curve_coefficients
            .as_ref()
        {
            lines.extend([
                field("neurons_fund_economics.neurons_fund_matched_funding_curve_coefficients.contribution_threshold_xdr", &(decimal_text(curve.contribution_threshold_xdr.as_ref()))),
                field("neurons_fund_economics.neurons_fund_matched_funding_curve_coefficients.one_third_participation_milestone_xdr", &(decimal_text(curve.one_third_participation_milestone_xdr.as_ref()))),
                field("neurons_fund_economics.neurons_fund_matched_funding_curve_coefficients.full_participation_milestone_xdr", &(decimal_text(curve.full_participation_milestone_xdr.as_ref()))),
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
            field(
                "voting_power_economics.start_reducing_voting_power_after_seconds",
                &(optional_u64_text(voting.start_reducing_voting_power_after_seconds)),
            ),
            field(
                "voting_power_economics.clear_following_after_seconds",
                &(optional_u64_text(voting.clear_following_after_seconds)),
            ),
            field(
                "voting_power_economics.neuron_minimum_dissolve_delay_to_vote_seconds",
                &(optional_u64_text(voting.neuron_minimum_dissolve_delay_to_vote_seconds)),
            ),
        ]);
    } else {
        lines.push("voting_power_economics: -".to_string());
    }
    lines.join("\n")
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
            value.human_readable.as_deref().map_or_else(
                || "-".to_string(),
                |value| terminal_text(value).into_owned(),
            )
        },
    )
}
