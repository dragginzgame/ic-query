use super::super::super::SnsNeuronsReport;
use super::super::common::{
    optional_e8s_decimal_text, optional_text, push_report_provenance_lines,
};
use super::common::neuron_id_for_list;
use crate::{
    nns::render::yes_no,
    table::{ColumnAlign, render_table},
    token_amount::e8s_decimal_text,
};

#[must_use]
pub fn sns_neurons_report_text(report: &SnsNeuronsReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
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
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]);
    if !report.neurons.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "NEURON_ID",
                "STAKE",
                "MATURITY",
                "STAKED_MATURITY",
                "CREATED_AT",
            ],
            &report
                .neurons
                .iter()
                .map(|neuron| {
                    [
                        neuron_id_for_list(&neuron.neuron_id, report.verbose),
                        e8s_decimal_text(neuron.cached_neuron_stake_e8s),
                        e8s_decimal_text(neuron.maturity_e8s_equivalent),
                        optional_e8s_decimal_text(neuron.staked_maturity_e8s_equivalent),
                        neuron.created_at.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
    }
    lines.join("\n")
}
