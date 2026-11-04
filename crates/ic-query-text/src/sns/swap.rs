//! Module: sns::swap
//!
//! Responsibility: render bounded SNS swap reports as human-facing text.
//! Does not own: swap calls, report construction, source validation, or JSON output.
//! Boundary: presents native raw fields, query guarantees, and typed gaps without hiding absence.

use crate::sns::labels::sns_swap_lifecycle_name;
use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::text_value::optional_u64_text;
use crate::text_value::yes_no;
use ic_query::sns::SnsSwapDerivedState;
use ic_query::sns::SnsSwapLifecycle;
use ic_query::sns::SnsSwapQueryGap;
use ic_query::sns::SnsSwapReport;
use ic_query::sns::SnsSwapSaleParameters;

/// Render one SNS swap report as human-facing text.
#[must_use]
pub fn sns_swap_report_text(report: &SnsSwapReport) -> String {
    let mut lines = swap_header_lines(report);
    push_component_section(
        &mut lines,
        "lifecycle",
        report.swap.lifecycle.as_ref().map(lifecycle_text),
    );
    push_component_section(
        &mut lines,
        "sale_parameters",
        report
            .swap
            .sale_parameters
            .as_ref()
            .map(sale_parameters_text),
    );
    push_component_section(
        &mut lines,
        "derived_state",
        report.swap.derived_state.as_ref().map(derived_state_text),
    );
    if !report.swap.gaps.is_empty() {
        push_component_section(&mut lines, "gaps", Some(gaps_text(&report.swap.gaps)));
    }
    lines.join("\n")
}

fn swap_header_lines(report: &SnsSwapReport) -> Vec<String> {
    vec![
        field("network", &report.context.network),
        field("sns_id", &(report.context.id)),
        field("name", &report.context.name),
        field("root_canister_id", &(report.context.root_canister_id)),
        field("swap_canister_id", &(report.swap.swap_canister_id)),
        field("component_query_count", &(report.component_query_count)),
        field(
            "successful_component_query_count",
            &(report.successful_component_query_count),
        ),
        field("component_gap_count", &(report.component_gap_count)),
        field(
            "point_in_time_guaranteed",
            &(yes_no(report.swap.point_in_time_guaranteed)),
        ),
        field("lifecycle_method", &(report.swap.lifecycle_method.as_str())),
        field(
            "sale_parameters_method",
            &(report.swap.sale_parameters_method.as_str()),
        ),
        field(
            "derived_state_method",
            &(report.swap.derived_state_method.as_str()),
        ),
        field(
            "sns_wasm_canister_id",
            &(report.context.sns_wasm_canister_id),
        ),
        field("fetched_at", &report.context.fetched_at),
        field("source_endpoint", &report.context.source_endpoint),
    ]
}

fn push_component_section(lines: &mut Vec<String>, label: &str, text: Option<String>) {
    lines.push(String::new());
    lines.push(format!("{label}:"));
    lines.push(text.unwrap_or_else(|| "-".to_string()));
}

fn lifecycle_text(lifecycle: &SnsSwapLifecycle) -> String {
    key_value_table(&[
        row(
            "lifecycle",
            lifecycle
                .lifecycle
                .map_or_else(|| "-".to_string(), |value| value.to_string()),
        ),
        row(
            "lifecycle_name",
            sns_swap_lifecycle_name(lifecycle.lifecycle)
                .unwrap_or("-")
                .to_string(),
        ),
        row(
            "decentralization_sale_open_timestamp_seconds",
            optional_u64_text(lifecycle.decentralization_sale_open_timestamp_seconds),
        ),
        row(
            "decentralization_swap_termination_timestamp_seconds",
            optional_u64_text(lifecycle.decentralization_swap_termination_timestamp_seconds),
        ),
    ])
}

fn sale_parameters_text(params: &SnsSwapSaleParameters) -> String {
    let basket = params.neuron_basket_construction_parameters.as_ref();
    key_value_table(&[
        row("min_icp_e8s", params.min_icp_e8s.to_string()),
        row("max_icp_e8s", params.max_icp_e8s.to_string()),
        row(
            "min_direct_participation_icp_e8s",
            optional_u64_text(params.min_direct_participation_icp_e8s),
        ),
        row(
            "max_direct_participation_icp_e8s",
            optional_u64_text(params.max_direct_participation_icp_e8s),
        ),
        row("sns_token_e8s", params.sns_token_e8s.to_string()),
        row("min_participants", params.min_participants.to_string()),
        row(
            "min_participant_icp_e8s",
            params.min_participant_icp_e8s.to_string(),
        ),
        row(
            "max_participant_icp_e8s",
            params.max_participant_icp_e8s.to_string(),
        ),
        row(
            "swap_due_timestamp_seconds",
            params.swap_due_timestamp_seconds.to_string(),
        ),
        row(
            "sale_delay_seconds",
            optional_u64_text(params.sale_delay_seconds),
        ),
        row(
            "neuron_basket_count",
            optional_u64_text(basket.map(|basket| basket.count)),
        ),
        row(
            "neuron_basket_dissolve_delay_interval_seconds",
            optional_u64_text(basket.map(|basket| basket.dissolve_delay_interval_seconds)),
        ),
    ])
}

fn derived_state_text(state: &SnsSwapDerivedState) -> String {
    key_value_table(&[
        row(
            "sns_tokens_per_icp",
            state
                .sns_tokens_per_icp
                .map_or_else(|| "-".to_string(), |value| value.to_string()),
        ),
        row(
            "buyer_total_icp_e8s",
            optional_u64_text(state.buyer_total_icp_e8s),
        ),
        row(
            "direct_participation_icp_e8s",
            optional_u64_text(state.direct_participation_icp_e8s),
        ),
        row(
            "neurons_fund_participation_icp_e8s",
            optional_u64_text(state.neurons_fund_participation_icp_e8s),
        ),
        row(
            "direct_participant_count",
            optional_u64_text(state.direct_participant_count),
        ),
        row(
            "cf_participant_count",
            optional_u64_text(state.cf_participant_count),
        ),
        row("cf_neuron_count", optional_u64_text(state.cf_neuron_count)),
    ])
}

fn gaps_text(gaps: &[SnsSwapQueryGap]) -> String {
    table(
        &["COMPONENT", "METHOD", "REASON"],
        &gaps
            .iter()
            .map(|gap| {
                [
                    gap.component.as_str().to_string(),
                    gap.method.as_str().to_string(),
                    terminal_text(&gap.reason).into_owned(),
                ]
            })
            .collect::<Vec<_>>(),
        &[Alignment::Left; 3],
    )
}

fn key_value_table(rows: &[[String; 2]]) -> String {
    table(
        &["FIELD", "VALUE"],
        rows,
        &[Alignment::Left, Alignment::Right],
    )
}

fn row(field: &str, value: String) -> [String; 2] {
    [field.to_string(), value]
}
