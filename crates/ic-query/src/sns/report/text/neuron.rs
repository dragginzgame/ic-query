//! Module: sns::report::text::neuron
//!
//! Responsibility: render one exact SNS neuron detail report for humans.
//! Does not own: live fetching, source validation, or JSON serialization.
//! Boundary: formats raw neuron, permission, disbursement, and followee evidence.

use crate::{
    sns::report::{
        SnsNeuronAccount, SnsNeuronDetail, SnsNeuronDetailReport, SnsNeuronDissolveState,
        text::common::{optional_bool_text, optional_e8s_text},
    },
    table::{ColumnAlign, render_table},
    text_value::{optional_text, optional_u64_text, sanitize_text},
    token_amount::e8s_decimal_text,
};

/// Render one exact SNS neuron detail report as human-readable text.
#[must_use]
pub fn sns_neuron_detail_report_text(report: &SnsNeuronDetailReport) -> String {
    let mut sections = vec![neuron_header_lines(report).join("\n")];
    sections.extend(permission_table(&report.detail));
    sections.extend(maturity_disbursement_table(&report.detail));
    sections.extend(legacy_followee_table(&report.detail));
    sections.extend(topic_followee_table(&report.detail));
    sections.join("\n\n")
}

fn neuron_header_lines(report: &SnsNeuronDetailReport) -> Vec<String> {
    let detail = &report.detail;
    let neuron = &detail.neuron;
    vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", sanitize_text(&report.name)),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("neuron_id: {}", report.neuron_id),
        format!("data_source: {}", report.data_source),
        format!(
            "maturity_mint_conversion_observed_disabled: {}",
            detail.maturity_mint_conversion_observed_disabled.as_str()
        ),
        format!(
            "manual_maturity_staking_observed_disabled: {}",
            detail.manual_maturity_staking_observed_disabled.as_str()
        ),
        format!(
            "cached_neuron_stake: {}",
            e8s_decimal_text(neuron.cached_neuron_stake_e8s)
        ),
        format!("neuron_fees: {}", e8s_decimal_text(neuron.neuron_fees_e8s)),
        format!(
            "maturity: {}",
            e8s_decimal_text(neuron.maturity_e8s_equivalent)
        ),
        format!(
            "staked_maturity: {}",
            optional_e8s_text(neuron.staked_maturity_e8s_equivalent)
        ),
        format!(
            "auto_stake_maturity: {}",
            optional_bool_text(neuron.auto_stake_maturity)
        ),
        format!(
            "dissolve_state: {}",
            dissolve_state_text(neuron.dissolve_state)
        ),
        format!(
            "voting_power_percentage_multiplier: {}",
            neuron.voting_power_percentage_multiplier
        ),
        format!("created_at: {}", sanitize_text(&neuron.created_at)),
        format!(
            "source_nns_neuron_id: {}",
            optional_u64_text(neuron.source_nns_neuron_id)
        ),
        format!(
            "vesting_period_seconds: {}",
            optional_u64_text(neuron.vesting_period_seconds)
        ),
        format!("permission_entry_count: {}", detail.permissions.len()),
        format!(
            "pending_maturity_disbursement_count: {}",
            detail.disburse_maturity_in_progress.len()
        ),
        format!("legacy_following_entry_count: {}", detail.followees.len()),
        format!(
            "topic_following_entry_count: {}",
            detail.topic_followees.as_ref().map_or(0, Vec::len)
        ),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ]
}

fn permission_table(detail: &SnsNeuronDetail) -> Option<String> {
    (!detail.permissions.is_empty()).then(|| {
        render_table(
            &["PRINCIPAL", "PERMISSIONS"],
            &detail
                .permissions
                .iter()
                .map(|permission| {
                    [
                        optional_text(permission.principal.as_ref()),
                        permission
                            .permission_types
                            .iter()
                            .map(|value| format!("{}:{}", value.code, value.name))
                            .collect::<Vec<_>>()
                            .join(","),
                    ]
                })
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left],
        )
    })
}

fn maturity_disbursement_table(detail: &SnsNeuronDetail) -> Option<String> {
    (!detail.disburse_maturity_in_progress.is_empty()).then(|| {
        render_table(
            &["SCHEDULED_AT", "AMOUNT", "DESTINATION", "FINALIZE_AT"],
            &detail
                .disburse_maturity_in_progress
                .iter()
                .map(|disbursement| {
                    [
                        disbursement.timestamp_of_disbursement_seconds.to_string(),
                        e8s_decimal_text(disbursement.amount_e8s),
                        account_text(disbursement.account_to_disburse_to.as_ref()),
                        optional_u64_text(disbursement.finalize_disbursement_timestamp_seconds),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Right,
            ],
        )
    })
}

fn legacy_followee_table(detail: &SnsNeuronDetail) -> Option<String> {
    (!detail.followees.is_empty()).then(|| {
        render_table(
            &["FUNCTION_ID", "FOLLOWEE_NEURON_IDS"],
            &detail
                .followees
                .iter()
                .map(|followees| {
                    [
                        followees.function_id.to_string(),
                        followees.followee_neuron_ids.join(","),
                    ]
                })
                .collect::<Vec<_>>(),
            &[ColumnAlign::Right, ColumnAlign::Left],
        )
    })
}

fn topic_followee_table(detail: &SnsNeuronDetail) -> Option<String> {
    detail
        .topic_followees
        .as_ref()
        .filter(|topic_followees| !topic_followees.is_empty())
        .map(|topic_followees| {
            render_table(
                &["TOPIC_CODE", "TOPIC", "FOLLOWEES"],
                &topic_followees
                    .iter()
                    .map(|topic| {
                        [
                            topic.topic_code.to_string(),
                            optional_text(topic.topic.as_ref()),
                            topic
                                .followees
                                .iter()
                                .map(|followee| {
                                    let id = optional_text(followee.neuron_id.as_ref());
                                    let alias = optional_text(followee.alias.as_ref());
                                    format!("{id}:{alias}")
                                })
                                .collect::<Vec<_>>()
                                .join(","),
                        ]
                    })
                    .collect::<Vec<_>>(),
                &[ColumnAlign::Right, ColumnAlign::Left, ColumnAlign::Left],
            )
        })
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

fn account_text(account: Option<&SnsNeuronAccount>) -> String {
    account.map_or_else(
        || "-".to_string(),
        |account| {
            format!(
                "{}:{}",
                optional_text(account.owner.as_ref()),
                optional_text(account.subaccount_hex.as_ref())
            )
        },
    )
}
