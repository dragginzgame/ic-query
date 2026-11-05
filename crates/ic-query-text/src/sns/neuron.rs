//! Render exact neuron stake and principal permission evidence.
use crate::text_value::table;
use ic_host_text::table::Alignment;
use ic_query::sns::{SnsNeuronDetailReport, sns_neuron_permission_name};
/// Render one exact neuron without inferring purchase eligibility.
#[must_use]
pub fn sns_neuron_detail_report_text(report: &SnsNeuronDetailReport) -> String {
    let mut lines = super::common::sns_identity_lines(&report.context);
    lines.extend(super::common::sns_collection_lines(&report.context));
    let neuron = &report.neuron;
    lines.extend(crate::text_value::report_fields!(report;
        neuron_id,
        data_source,
        cached_neuron_stake_e8s = neuron.cached_neuron_stake_e8s,
        neuron_fees_e8s = neuron.neuron_fees_e8s,
        dissolve_state = super::common::dissolve_state_text(neuron.dissolve_state),
    ));
    if !neuron.permissions.is_empty() {
        lines.push(String::new());
        lines.push(table(
            &["PRINCIPAL", "PERMISSIONS"],
            &neuron
                .permissions
                .iter()
                .map(|p| {
                    [
                        p.principal.clone(),
                        p.permission_types
                            .iter()
                            .map(|code| format!("{code}:{}", sns_neuron_permission_name(*code)))
                            .collect::<Vec<_>>()
                            .join(","),
                    ]
                })
                .collect::<Vec<_>>(),
            &[Alignment::Left, Alignment::Left],
        ));
    }
    lines.join("\n")
}
