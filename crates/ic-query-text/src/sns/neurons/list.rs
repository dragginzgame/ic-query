//! Module: sns::neurons::list
//!
//! Responsibility: render SNS neuron list reports as text.
//! Does not own: neuron fetching, cache sorting, report construction, or JSON output.
//! Boundary: formats live or cache-backed neuron report rows for humans.

use crate::sns::common::sns_identity_lines;
use crate::text_value::field;
use crate::text_value::table;

use ic_host_text::table::Alignment;
use ic_host_text::timestamp::format_utc_timestamp_secs;

use crate::sns::common::dissolve_state_text;
use crate::sns::common::neuron_id_text;
use crate::sns::common::optional_e8s_text;
use crate::sns::common::optional_text;
use crate::sns::common::push_report_provenance_lines;
use crate::text_value::optional_bool_text;
use crate::text_value::yes_no;
use crate::token_amount::e8s_decimal_text;
use ic_query::sns::SnsNeuronsReport;

#[must_use]
/// Render the SNS neurons report as human-facing text.
pub fn sns_neurons_report_text(report: &SnsNeuronsReport, options: crate::TextOptions) -> String {
    let mut lines = sns_identity_lines(&report.context);
    lines.extend([
        field("requested_limit", &(report.requested_limit)),
        field(
            "owner_principal_id",
            &(optional_text(report.owner_principal_id.as_ref())),
        ),
        field("verbose", &(yes_no(options.verbose))),
    ]);
    push_report_provenance_lines(&mut lines, &report.provenance);
    lines.extend([
        field("sort", &(report.sort)),
        field("total_neuron_count", &(report.total_neuron_count)),
        field("neuron_count", &(report.neurons.len())),
    ]);
    lines.extend(crate::sns::common::sns_collection_lines(&report.context));
    if !report.neurons.is_empty() {
        lines.push(String::new());
        lines.push(table(
            &[
                "NEURON_ID",
                "PRINCIPALS",
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
                        neuron_id_text(&neuron.neuron_id, options.verbose),
                        neuron
                            .permissions
                            .iter()
                            .map(|permission| permission.principal.as_str())
                            .collect::<Vec<_>>()
                            .join(","),
                        e8s_decimal_text(neuron.cached_neuron_stake_e8s),
                        e8s_decimal_text(neuron.neuron_fees_e8s),
                        e8s_decimal_text(neuron.maturity_e8s_equivalent),
                        optional_e8s_text(neuron.staked_maturity_e8s_equivalent),
                        dissolve_state_text(neuron.dissolve_state),
                        optional_bool_text(neuron.auto_stake_maturity).to_string(),
                        neuron.voting_power_percentage_multiplier.to_string(),
                        format_utc_timestamp_secs(neuron.created_timestamp_seconds),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Left,
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
        ));
    }
    lines.join("\n")
}
