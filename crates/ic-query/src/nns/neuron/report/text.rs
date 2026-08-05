//! Module: nns::neuron::report::text
//!
//! Responsibility: render public NNS neuron reports as human-readable text.
//! Does not own: live Governance calls, cache IO, or JSON serialization.
//! Boundary: formats raw public neuron values without changing report data.

#[cfg(feature = "nns-host")]
use super::cache::{NnsNeuronCacheStatusReport, NnsNeuronRefreshReport};
use super::model::{NnsNeuronInfoReport, NnsNeuronListReport, NnsNeuronRow};
#[cfg(feature = "nns-host")]
use crate::nns::NnsGovernanceRefreshAttemptStatus;
use crate::{
    duration::display_duration_seconds,
    table::{ColumnAlign, render_table},
    text_value::{optional_u64_text, sanitize_text, yes_no},
    token_amount::e8s_decimal_text,
};

/// Render one public NNS neuron-index page.
#[must_use]
pub fn nns_neuron_list_report_text(report: &NnsNeuronListReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("governance_canister_id: {}", report.governance_canister_id),
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
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
    ];
    if let Some(cache_path) = report.cache_path.as_deref() {
        lines.push(format!("cache_path: {}", sanitize_text(cache_path)));
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
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("from_cache: {}", yes_no(report.from_cache)),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
    ];
    if let Some(cache_path) = report.cache_path.as_deref() {
        lines.push(format!("cache_path: {}", sanitize_text(cache_path)));
    }
    lines.push(String::new());
    lines.extend(neuron_detail_lines(&report.neuron));
    lines.join("\n")
}

/// Render one complete NNS neuron snapshot refresh report.
#[cfg(feature = "nns-host")]
#[must_use]
pub fn nns_neuron_refresh_report_text(report: &NnsNeuronRefreshReport) -> String {
    [
        format!("network: {}", sanitize_text(&report.network)),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("neuron_count: {}", report.neuron_count),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("complete: {}", yes_no(report.complete)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!(
            "attempt_finalized: {}",
            yes_no(report.attempt_finalization_error.is_none())
        ),
        format!(
            "attempt_finalization_error: {}",
            sanitize_text(report.attempt_finalization_error.as_deref().unwrap_or("-"))
        ),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
        format!("cache_path: {}", sanitize_text(&report.cache_path)),
        format!(
            "refresh_attempt_path: {}",
            sanitize_text(&report.refresh_attempt_path)
        ),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
    ]
    .join("\n")
}

/// Render local NNS neuron cache and refresh-attempt status.
#[cfg(feature = "nns-host")]
#[must_use]
pub fn nns_neuron_cache_status_report_text(report: &NnsNeuronCacheStatusReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("cache_root: {}", sanitize_text(&report.cache_root)),
        format!("found: {}", yes_no(report.found)),
        format!(
            "expected_cache_path: {}",
            sanitize_text(&report.expected_cache_path)
        ),
        format!(
            "refresh_attempt_path: {}",
            sanitize_text(&report.refresh_attempt_path)
        ),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("cache_status: {}", cache.cache_status),
            format!(
                "cache_error: {}",
                sanitize_text(cache.cache_error.as_deref().unwrap_or("-"))
            ),
            format!("complete: {}", yes_no(cache.complete)),
            format!(
                "point_in_time_guaranteed: {}",
                yes_no(cache.point_in_time_guaranteed)
            ),
            format!("row_count: {}", cache.row_count),
            format!("page_count: {}", cache.page_count),
            format!("page_size: {}", cache.page_size),
            format!("fetched_at: {}", sanitize_text(&cache.fetched_at)),
            format!("source_endpoint: {}", sanitize_text(&cache.source_endpoint)),
        ]);
    }
    lines.extend(attempt_lines(report.latest_attempt.as_ref()));
    lines.join("\n")
}

fn neuron_table(neurons: &[NnsNeuronRow]) -> String {
    render_table(
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
                    neuron.state_text.to_string(),
                    neuron.visibility_text.to_string(),
                    e8s_decimal_text(neuron.stake_e8s),
                    display_duration_seconds(neuron.dissolve_delay_seconds),
                    optional_u64_text(neuron.deciding_voting_power),
                    neuron
                        .known_neuron_data
                        .as_ref()
                        .map_or_else(|| "-".to_string(), |known| sanitize_text(&known.name)),
                ]
            })
            .collect::<Vec<_>>(),
        &[
            ColumnAlign::Right,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Right,
            ColumnAlign::Right,
            ColumnAlign::Right,
            ColumnAlign::Left,
        ],
    )
}

fn neuron_detail_lines(neuron: &NnsNeuronRow) -> Vec<String> {
    let known = neuron.known_neuron_data.as_ref();
    vec![
        format!("neuron_id: {}", neuron.neuron_id),
        format!("state: {} ({})", neuron.state_text, neuron.state),
        format!(
            "visibility: {} ({})",
            neuron.visibility_text,
            neuron
                .visibility
                .map_or_else(|| "-".to_string(), |value| value.to_string())
        ),
        format!(
            "neuron_type: {} ({})",
            neuron.neuron_type_text,
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
            known.map_or_else(|| "-".to_string(), |known| sanitize_text(&known.name))
        ),
        format!(
            "known_description: {}",
            known
                .and_then(|known| known.description.as_deref())
                .map_or_else(|| "-".to_string(), sanitize_text)
        ),
        format!(
            "known_links: {}",
            known.map_or_else(
                || "-".to_string(),
                |known| known
                    .links
                    .iter()
                    .map(|link| sanitize_text(link))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ),
        format!("recent_ballot_count: {}", neuron.recent_ballots.len()),
    ]
}

#[cfg(feature = "nns-host")]
fn attempt_lines(attempt: Option<&NnsGovernanceRefreshAttemptStatus>) -> Vec<String> {
    let Some(attempt) = attempt else {
        return vec!["latest_attempt: -".to_string()];
    };
    vec![
        format!("attempt_status: {}", attempt.status),
        format!("attempt_started_at: {}", sanitize_text(&attempt.started_at)),
        format!("attempt_updated_at: {}", sanitize_text(&attempt.updated_at)),
        format!("attempt_page_size: {}", attempt.page_size),
        format!("attempt_pages_fetched: {}", attempt.pages_fetched),
        format!("attempt_rows_fetched: {}", attempt.rows_fetched),
        format!(
            "attempt_last_cursor: {}",
            sanitize_text(attempt.last_cursor.as_deref().unwrap_or("-"))
        ),
        format!(
            "attempt_last_error: {}",
            sanitize_text(attempt.last_error.as_deref().unwrap_or("-"))
        ),
    ]
}
