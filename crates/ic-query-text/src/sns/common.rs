//! Module: sns::common
//!
//! Responsibility: group shared SNS text rendering helpers.
//! Does not own: report DTOs, tables, cache behavior, or source reads.
//! Boundary: exposes small formatting helpers used by text report leaves.

#[cfg(feature = "sns-host")]
pub(super) use crate::report::push_report_provenance_lines;
use crate::text_value::field;
#[cfg(feature = "sns-host")]
use ic_host_text::escape::terminal_text;

#[cfg(feature = "sns-host")]
use crate::token_amount::e8s_decimal_text;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsCacheSummary;
use ic_query::sns::SnsNeuronDissolveState;

#[cfg(feature = "sns-host")]
const COMPACT_NEURON_ID_CHARS: usize = 8;

pub(super) fn sns_identity_lines(context: &ic_query::sns::SnsReportContext) -> Vec<String> {
    vec![
        field("network", &context.instance.network),
        field("governance_canister_id", &context.governance_canister_id),
    ]
}
pub(super) fn sns_collection_lines(context: &ic_query::sns::SnsReportContext) -> [String; 3] {
    [
        field("fetched_at", &context.instance.fetched_at),
        field("source_endpoint", &context.instance.source_endpoint),
        field("fetched_by", &context.instance.fetched_by),
    ]
}

#[cfg(feature = "sns-host")]
pub(super) use crate::text_value::optional_text;

#[cfg(feature = "sns-host")]
pub(super) fn optional_e8s_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), e8s_decimal_text)
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
