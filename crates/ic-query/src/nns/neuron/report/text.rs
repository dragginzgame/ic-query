//! Module: nns::neuron::report::text
//!
//! Responsibility: render public NNS neuron reports as human-readable text.
//! Does not own: live Governance calls, cache IO, or JSON serialization.
//! Boundary: formats raw public neuron values without changing report data.

use crate::text_value::{REPORT_TABLE_OPTIONS, push_titled_table_section};
use ic_host_text::duration::display_duration_seconds;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use super::classification::{NnsNeuronState, NnsNeuronType, NnsNeuronVisibility};
use super::distribution::NnsNeuronDistributionReport;
use super::model::{NnsNeuronInfoReport, NnsNeuronListReport, NnsNeuronRow};
use crate::{
    nns::governance::{governance_collection_context_lines, governance_context_lines},
    text_value::{optional_u64_text, yes_no},
    token_amount::e8s_decimal_text,
};

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
        &report.network,
        &report.governance_canister_id,
        &report.collection_started_at,
        &report.collection_updated_at,
        &report.source,
    );
    lines.extend([
        format!("collection_page_count: {}", report.collection_page_count),
        format!("collected_neuron_count: {}", report.collected_neuron_count),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!(
            "earliest_retrieved_at_timestamp_seconds: {}",
            optional_u64_text(report.earliest_retrieved_at_timestamp_seconds)
        ),
        format!(
            "latest_retrieved_at_timestamp_seconds: {}",
            optional_u64_text(report.latest_retrieved_at_timestamp_seconds)
        ),
        format!(
            "total_effective_stake_e8s: {}",
            report.total_effective_stake_e8s
        ),
        format!(
            "total_effective_stake_icp: {}",
            e8s_decimal_text(report.total_effective_stake_e8s)
        ),
        format!(
            "reported_staked_maturity_neuron_count: {}",
            report.reported_staked_maturity_neuron_count
        ),
        format!(
            "unreported_staked_maturity_neuron_count: {}",
            report.unreported_staked_maturity_neuron_count
        ),
        format!(
            "total_reported_staked_maturity_e8s_equivalent: {}",
            report.total_reported_staked_maturity_e8s_equivalent
        ),
        format!(
            "total_reported_staked_maturity_icp: {}",
            e8s_decimal_text(report.total_reported_staked_maturity_e8s_equivalent)
        ),
        format!(
            "reported_deciding_voting_power_neuron_count: {}",
            report.reported_deciding_voting_power_neuron_count
        ),
        format!(
            "unreported_deciding_voting_power_neuron_count: {}",
            report.unreported_deciding_voting_power_neuron_count
        ),
        format!(
            "total_reported_deciding_voting_power: {}",
            report.total_reported_deciding_voting_power
        ),
        format!(
            "reported_potential_voting_power_neuron_count: {}",
            report.reported_potential_voting_power_neuron_count
        ),
        format!(
            "unreported_potential_voting_power_neuron_count: {}",
            report.unreported_potential_voting_power_neuron_count
        ),
        format!(
            "total_reported_potential_voting_power: {}",
            report.total_reported_potential_voting_power
        ),
        format!(
            "known_neuron_metadata_count: {}",
            report.known_neuron_metadata_count
        ),
        format!(
            "neurons_fund_join_timestamp_present_count: {}",
            report.neurons_fund_join_timestamp_present_count
        ),
    ]);
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
        render(
            &headers,
            &rows,
            &[
                Alignment::Right,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
            ],
            REPORT_TABLE_OPTIONS,
        )
    })
}

/// Render one public NNS neuron-index page.
#[must_use]
pub fn nns_neuron_list_report_text(report: &NnsNeuronListReport) -> String {
    let mut lines = governance_context_lines(&report.context);
    lines.extend([
        format!("from_cache: {}", yes_no(report.from_cache)),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "exclusive_start_neuron_id: {}",
            optional_u64_text(report.exclusive_start_neuron_id)
        ),
        format!(
            "next_start_neuron_id: {}",
            optional_u64_text(report.next_start_neuron_id)
        ),
        format!(
            "total_neuron_count: {}",
            report
                .total_neuron_count
                .map_or_else(|| "-".to_string(), |count| count.to_string())
        ),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!("returned_neuron_count: {}", report.returned_neuron_count),
        format!("verbose: {}", yes_no(report.verbose)),
    ]);
    if let Some(cache_path) = report.cache_path.as_deref() {
        lines.push(format!("cache_path: {}", terminal_text(cache_path)));
    }
    if !report.neurons.is_empty() {
        lines.push(String::new());
        lines.push(neuron_table(&report.neurons));
    }
    if report.verbose {
        for neuron in &report.neurons {
            lines.push(String::new());
            lines.extend(neuron_detail_lines(neuron));
        }
    }
    lines.join("\n")
}

/// Render one public NNS neuron detail report.
#[must_use]
pub fn nns_neuron_info_report_text(report: &NnsNeuronInfoReport) -> String {
    let mut lines = governance_context_lines(&report.context);
    lines.extend([format!("from_cache: {}", yes_no(report.from_cache))]);
    if let Some(cache_path) = report.cache_path.as_deref() {
        lines.push(format!("cache_path: {}", terminal_text(cache_path)));
    }
    lines.push(String::new());
    lines.extend(neuron_detail_lines(&report.neuron));
    lines.join("\n")
}

fn neuron_table(neurons: &[NnsNeuronRow]) -> String {
    render(
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
        REPORT_TABLE_OPTIONS,
    )
}

fn neuron_detail_lines(neuron: &NnsNeuronRow) -> Vec<String> {
    let known = neuron.known_neuron_data.as_ref();
    vec![
        format!("neuron_id: {}", neuron.neuron_id),
        format!(
            "state: {} ({})",
            NnsNeuronState::from_code(neuron.state),
            neuron.state
        ),
        format!(
            "visibility: {} ({})",
            NnsNeuronVisibility::from_code(neuron.visibility),
            neuron
                .visibility
                .map_or_else(|| "-".to_string(), |value| value.to_string())
        ),
        format!(
            "neuron_type: {} ({})",
            NnsNeuronType::from_code(neuron.neuron_type),
            neuron
                .neuron_type
                .map_or_else(|| "-".to_string(), |value| value.to_string())
        ),
        format!("stake_e8s: {}", neuron.stake_e8s),
        format!("stake_icp: {}", e8s_decimal_text(neuron.stake_e8s)),
        format!(
            "staked_maturity_e8s_equivalent: {}",
            optional_u64_text(neuron.staked_maturity_e8s_equivalent)
        ),
        format!(
            "dissolve_delay: {}",
            display_duration_seconds(neuron.dissolve_delay_seconds)
        ),
        format!("age: {}", display_duration_seconds(neuron.age_seconds)),
        format!(
            "created_timestamp_seconds: {}",
            neuron.created_timestamp_seconds
        ),
        format!(
            "retrieved_at_timestamp_seconds: {}",
            neuron.retrieved_at_timestamp_seconds
        ),
        format!("voting_power: {}", neuron.voting_power),
        format!(
            "deciding_voting_power: {}",
            optional_u64_text(neuron.deciding_voting_power)
        ),
        format!(
            "potential_voting_power: {}",
            optional_u64_text(neuron.potential_voting_power)
        ),
        format!(
            "voting_power_refreshed_timestamp_seconds: {}",
            optional_u64_text(neuron.voting_power_refreshed_timestamp_seconds)
        ),
        format!(
            "joined_community_fund_timestamp_seconds: {}",
            optional_u64_text(neuron.joined_community_fund_timestamp_seconds)
        ),
        format!(
            "eight_year_gang_bonus_base_e8s: {}",
            optional_u64_text(neuron.eight_year_gang_bonus_base_e8s)
        ),
        format!(
            "known_name: {}",
            known.map_or_else(
                || "-".to_string(),
                |known| terminal_text(&known.name).into_owned()
            )
        ),
        format!(
            "known_description: {}",
            known
                .and_then(|known| known.description.as_deref())
                .map_or_else(
                    || "-".to_string(),
                    |value| terminal_text(value).into_owned()
                )
        ),
        format!(
            "known_links: {}",
            known.map_or_else(
                || "-".to_string(),
                |known| known
                    .links
                    .iter()
                    .map(|link| terminal_text(link))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ),
        format!("recent_ballot_count: {}", neuron.recent_ballots.len()),
    ]
}
