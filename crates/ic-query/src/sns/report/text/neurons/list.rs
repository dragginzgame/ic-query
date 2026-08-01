//! Module: sns::report::text::neurons::list
//!
//! Responsibility: render SNS neuron list reports as text.
//! Does not own: neuron fetching, cache sorting, report construction, or JSON output.
//! Boundary: formats live or cache-backed neuron report rows for humans.

use crate::{
    sns::report::{
        SnsNeuronDissolveState, SnsNeuronsReport,
        text::common::{
            neuron_id_text, optional_bool_text, optional_e8s_decimal_text, optional_text,
            push_report_provenance_lines,
        },
    },
    table::{ColumnAlign, render_table},
    text_value::{sanitize_text, yes_no},
    token_amount::e8s_decimal_text,
};

#[must_use]
pub fn sns_neurons_report_text(report: &SnsNeuronsReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", sanitize_text(&report.name)),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "owner_principal_id: {}",
            optional_text(report.owner_principal_id.as_ref())
        ),
        format!("verbose: {}", yes_no(report.verbose)),
    ];
    push_report_provenance_lines(
        &mut lines,
        &report.data_source,
        report.cache_path.as_deref(),
        report.cache_complete,
    );
    lines.extend([
        format!("sort: {}", report.sort),
        format!("total_neuron_count: {}", report.total_neuron_count),
        format!("neuron_count: {}", report.neuron_count),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ]);
    if !report.neurons.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "NEURON_ID",
                "STAKE",
                "FEES",
                "MATURITY",
                "STAKED_MATURITY",
                "DISSOLVE",
                "AUTO_STAKE",
                "VOTING_%",
                "CREATED_AT",
            ],
            &report
                .neurons
                .iter()
                .map(|neuron| {
                    [
                        neuron_id_text(&neuron.neuron_id, report.verbose),
                        e8s_decimal_text(neuron.cached_neuron_stake_e8s),
                        e8s_decimal_text(neuron.neuron_fees_e8s),
                        e8s_decimal_text(neuron.maturity_e8s_equivalent),
                        optional_e8s_decimal_text(neuron.staked_maturity_e8s_equivalent),
                        dissolve_state_text(neuron.dissolve_state),
                        optional_bool_text(neuron.auto_stake_maturity),
                        neuron.voting_power_percentage_multiplier.to_string(),
                        neuron.created_at.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
    }
    lines.join("\n")
}

fn dissolve_state_text(state: Option<SnsNeuronDissolveState>) -> String {
    state.map_or_else(
        || "-".to_string(),
        |state| match state {
            SnsNeuronDissolveState::DissolveDelaySeconds(seconds) => {
                format!("delay:{seconds}")
            }
            SnsNeuronDissolveState::WhenDissolvedTimestampSeconds(seconds) => {
                format!("dissolved_at:{seconds}")
            }
        },
    )
}
