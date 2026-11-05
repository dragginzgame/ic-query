//! Module: nns::neuron
//!
//! Responsibility: render public NNS neuron reports as human-readable text.
//! Does not own: live Governance calls, cache IO, or JSON serialization.
//! Boundary: formats raw public neuron values without changing report data.

use crate::text_value::field;
use crate::text_value::push_titled_table_section;
use crate::text_value::table;
use ic_host_text::duration::display_duration_seconds;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::nns::governance::collection_context_lines as governance_collection_context_lines;
use crate::nns::governance::context_lines as governance_context_lines;
use crate::text_value::optional_u64_text;
use crate::text_value::yes_no;
use crate::token_amount::e8s_decimal_text;
use ic_query::nns::neuron::NnsNeuronDistributionReport;
use ic_query::nns::neuron::NnsNeuronInfoReport;
use ic_query::nns::neuron::NnsNeuronListReport;
use ic_query::nns::neuron::NnsNeuronRow;
use ic_query::nns::neuron::NnsNeuronState;
use ic_query::nns::neuron::NnsNeuronType;
use ic_query::nns::neuron::NnsNeuronVisibility;

/// Render one portable public-neuron distribution without selecting a process output sink.
#[must_use]
pub fn nns_neuron_distribution_report_text(report: &NnsNeuronDistributionReport) -> String {
    let mut lines = distribution_preamble(report);
    push_titled_table_section(&mut lines, "states:", state_distribution_table(report));
    push_titled_table_section(
        &mut lines,
        "visibilities:",
        visibility_distribution_table(report),
    );
    push_titled_table_section(
        &mut lines,
        "neuron_types:",
        neuron_type_distribution_table(report),
    );
    lines.join("\n")
}

fn distribution_preamble(report: &NnsNeuronDistributionReport) -> Vec<String> {
    let mut lines = governance_collection_context_lines(
        &report.context.network,
        &report.context.governance_canister_id,
        &report.context.collection_started_at,
        &report.context.collection_updated_at,
        &report.context.source,
    );
    lines.extend(crate::text_value::report_fields!(report;
        collection_page_count = report.context.collection_page_count,
        collected_neuron_count,
        point_in_time_guaranteed = yes_no(report.context.point_in_time_guaranteed),
        earliest_retrieved_at_timestamp_seconds = optional_u64_text(report.earliest_retrieved_at_timestamp_seconds),
        latest_retrieved_at_timestamp_seconds = optional_u64_text(report.latest_retrieved_at_timestamp_seconds),
        total_effective_stake_e8s,
        total_effective_stake_icp = e8s_decimal_text(report.total_effective_stake_e8s),
        reported_staked_maturity_neuron_count,
        unreported_staked_maturity_neuron_count,
        total_reported_staked_maturity_e8s_equivalent,
        total_reported_staked_maturity_icp = e8s_decimal_text(report.total_reported_staked_maturity_e8s_equivalent),
        reported_deciding_voting_power_neuron_count,
        unreported_deciding_voting_power_neuron_count,
        total_reported_deciding_voting_power,
        reported_potential_voting_power_neuron_count,
        unreported_potential_voting_power_neuron_count,
        total_reported_potential_voting_power,
        known_neuron_metadata_count,
        neurons_fund_join_timestamp_present_count,
    ));
    lines
}

fn state_distribution_table(report: &NnsNeuronDistributionReport) -> Option<String> {
    distribution_table(
        ["STATE_CODE", "STATE", "NEURONS", "EFFECTIVE_STAKE_ICP"],
        report.state_distribution.iter().map(|row| {
            [
                row.state.to_string(),
                NnsNeuronState::from_code(row.state).to_string(),
                row.neuron_count.to_string(),
                e8s_decimal_text(row.effective_stake_e8s),
            ]
        }),
    )
}

fn visibility_distribution_table(report: &NnsNeuronDistributionReport) -> Option<String> {
    distribution_table(
        [
            "VISIBILITY_CODE",
            "VISIBILITY",
            "NEURONS",
            "EFFECTIVE_STAKE_ICP",
        ],
        report.visibility_distribution.iter().map(|row| {
            [
                row.visibility
                    .map_or_else(|| "-".to_string(), |value| value.to_string()),
                NnsNeuronVisibility::from_code(row.visibility).to_string(),
                row.neuron_count.to_string(),
                e8s_decimal_text(row.effective_stake_e8s),
            ]
        }),
    )
}

fn neuron_type_distribution_table(report: &NnsNeuronDistributionReport) -> Option<String> {
    distribution_table(
        [
            "NEURON_TYPE_CODE",
            "NEURON_TYPE",
            "NEURONS",
            "EFFECTIVE_STAKE_ICP",
        ],
        report.neuron_type_distribution.iter().map(|row| {
            [
                row.neuron_type
                    .map_or_else(|| "-".to_string(), |value| value.to_string()),
                NnsNeuronType::from_code(row.neuron_type).to_string(),
                row.neuron_count.to_string(),
                e8s_decimal_text(row.effective_stake_e8s),
            ]
        }),
    )
}

fn distribution_table(
    headers: [&str; 4],
    rows: impl Iterator<Item = [String; 4]>,
) -> Option<String> {
    let rows = rows.collect::<Vec<_>>();
    (!rows.is_empty()).then(|| {
        table(
            &headers,
            &rows,
            &[
                Alignment::Right,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
            ],
        )
    })
}

/// Render one public NNS neuron-index page.
#[must_use]
pub fn nns_neuron_list_report_text(
    report: &NnsNeuronListReport,
    options: crate::TextOptions,
) -> String {
    let mut lines = governance_context_lines(&report.context);
    lines.extend(crate::text_value::report_fields!(report;
        from_cache = yes_no(report.provenance.cache_path().is_some()),
        requested_limit,
        exclusive_start_neuron_id = optional_u64_text(report.exclusive_start_neuron_id),
        next_start_neuron_id = optional_u64_text(report.next_start_neuron_id),
        total_neuron_count = report
                .total_neuron_count
                .map_or_else(|| "-".to_string(), |count| count.to_string()),
        point_in_time_guaranteed = yes_no(report.point_in_time_guaranteed),
        returned_neuron_count = report.neurons.len(),
        verbose = yes_no(options.verbose),
    ));
    if let Some(cache_path) = report.provenance.cache_path() {
        lines.push(field("cache_path", &(cache_path)));
    }
    if !report.neurons.is_empty() {
        lines.push(String::new());
        lines.push(neuron_table(&report.neurons));
    }

    lines.join("\n")
}

/// Render one public NNS neuron detail report.
#[must_use]
pub fn nns_neuron_info_report_text(report: &NnsNeuronInfoReport) -> String {
    let mut lines = governance_context_lines(&report.context);
    lines.extend([field(
        "from_cache",
        &(yes_no(report.provenance.cache_path().is_some())),
    )]);
    if let Some(cache_path) = report.provenance.cache_path() {
        lines.push(field("cache_path", &(cache_path)));
    }
    lines.push(String::new());
    lines.push(neuron_table(std::slice::from_ref(&report.neuron)));
    lines.join("\n")
}

fn neuron_table(neurons: &[NnsNeuronRow]) -> String {
    table(
        &[
            "ID",
            "STATE",
            "VISIBILITY",
            "STAKE ICP",
            "DISSOLVE",
            "DECIDING VP",
            "NAME",
        ],
        &neurons
            .iter()
            .map(|neuron| {
                [
                    neuron.neuron_id.to_string(),
                    NnsNeuronState::from_code(neuron.state).to_string(),
                    NnsNeuronVisibility::from_code(neuron.visibility).to_string(),
                    e8s_decimal_text(neuron.stake_e8s),
                    display_duration_seconds(neuron.dissolve_delay_seconds),
                    optional_u64_text(neuron.deciding_voting_power),
                    neuron.known_neuron_data.as_ref().map_or_else(
                        || "-".to_string(),
                        |known| terminal_text(&known.name).into_owned(),
                    ),
                ]
            })
            .collect::<Vec<_>>(),
        &[
            Alignment::Right,
            Alignment::Left,
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Left,
        ],
    )
}
