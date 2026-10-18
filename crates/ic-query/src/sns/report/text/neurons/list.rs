//! Module: sns::report::text::neurons::list
//!
//! Responsibility: render SNS neuron list reports as text.
//! Does not own: neuron fetching, cache sorting, report construction, or JSON output.
//! Boundary: formats live or cache-backed neuron report rows for humans.

use crate::subnet_catalog::format_utc_timestamp_secs;
use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use crate::{
    sns::report::{
        SnsNeuronsReport,
        text::common::{
            dissolve_state_text, neuron_id_text, optional_bool_text, optional_e8s_text,
            optional_text, push_report_provenance_lines,
        },
    },
    text_value::yes_no,
    token_amount::e8s_decimal_text,
};

#[must_use]
pub fn sns_neurons_report_text(report: &SnsNeuronsReport) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", terminal_text(&report.name)),
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
        report.data_source,
        report.cache_path.as_deref(),
        report.cache_complete,
    );
    lines.extend([
        format!("sort: {}", report.sort),
        format!("total_neuron_count: {}", report.total_neuron_count),
        format!("neuron_count: {}", report.neuron_count),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
    ]);
    if !report.neurons.is_empty() {
        lines.push(String::new());
        lines.push(render(
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
                        optional_e8s_text(neuron.staked_maturity_e8s_equivalent),
                        dissolve_state_text(neuron.dissolve_state),
                        optional_bool_text(neuron.auto_stake_maturity),
                        neuron.voting_power_percentage_multiplier.to_string(),
                        format_utc_timestamp_secs(neuron.created_timestamp_seconds),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Left,
            ],
            REPORT_TABLE_OPTIONS,
        ));
    }
    lines.join("\n")
}
