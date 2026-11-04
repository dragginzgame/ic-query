//! Module: sns::common
//!
//! Responsibility: group shared SNS text rendering helpers.
//! Does not own: report DTOs, tables, cache behavior, or source reads.
//! Boundary: exposes small formatting helpers used by text report leaves.

pub(super) use crate::report::push_report_provenance_lines;
use crate::text_value::field;
use ic_host_text::duration::display_duration_seconds;
#[cfg(feature = "sns-host")]
use ic_host_text::escape::terminal_text;

use crate::token_amount::e8s_decimal_text;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsCacheSummary;
use ic_query::sns::SnsNeuronDissolveState;
use ic_query::sns::SnsNeuronPermissionList;

#[cfg(feature = "sns-host")]
const COMPACT_NEURON_ID_CHARS: usize = 8;

pub(super) fn sns_identity_lines(context: &ic_query::sns::SnsReportContext) -> Vec<String> {
    sns_identity_fields_lines(
        &context.instance.network,
        context.instance.id,
        &context.instance.name,
        &context.instance.root_canister_id,
        &context.governance_canister_id,
    )
}

pub(super) fn sns_identity_fields_lines(
    network: &str,
    id: usize,
    name: &str,
    root_canister_id: &str,
    governance_canister_id: &str,
) -> Vec<String> {
    vec![
        field("network", &(network)),
        field("sns_id", &(id)),
        field("name", &(name)),
        field("root_canister_id", &(root_canister_id)),
        field("governance_canister_id", &(governance_canister_id)),
    ]
}

pub(super) fn sns_collection_lines(context: &ic_query::sns::SnsReportContext) -> [String; 3] {
    [
        field(
            "sns_wasm_canister_id",
            &(context.instance.sns_wasm_canister_id),
        ),
        field("fetched_at", &context.instance.fetched_at),
        field("source_endpoint", &context.instance.source_endpoint),
    ]
}

#[cfg(feature = "sns-host")]
pub(super) use crate::text_value::optional_text;

pub(super) fn optional_e8s_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), e8s_decimal_text)
}

pub(super) fn optional_duration_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), display_duration_seconds)
}

pub(super) fn optional_percentage_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| format!("{value}%"))
}

pub(super) fn optional_basis_points_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), basis_points_text)
}

pub(super) fn optional_u32_text(value: Option<u32>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

pub(super) fn optional_permissions_text(value: Option<&SnsNeuronPermissionList>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |permissions| {
            permissions
                .permissions
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        },
    )
}

pub(super) fn comma_join_u64(values: &[u64]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(feature = "sns-host")]
pub(super) fn neuron_id_text(value: &str, verbose: bool) -> String {
    if verbose || value == "-" {
        value.to_string()
    } else {
        value.chars().take(COMPACT_NEURON_ID_CHARS).collect()
    }
}

#[cfg(feature = "sns-host")]
pub(super) fn push_cache_error_lines(lines: &mut Vec<String>, caches: &[SnsCacheSummary]) {
    lines.extend(caches.iter().filter_map(|cache| {
        cache.cache_error().map(|error| {
            format!(
                "cache_error: {}: {}",
                terminal_text(&cache.cache_path),
                terminal_text(error)
            )
        })
    }));
}

fn basis_points_text(value: u64) -> String {
    let whole = value / 100;
    let fractional = value % 100;
    format!("{whole}.{fractional:02}%")
}

pub(super) fn dissolve_state_text(state: Option<SnsNeuronDissolveState>) -> String {
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
