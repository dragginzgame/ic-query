//! Module: sns::neuron
//!
//! Responsibility: render one exact SNS neuron detail report for humans.
//! Does not own: live fetching, source validation, or JSON serialization.
//! Boundary: formats raw neuron, permission, disbursement, and followee evidence.

use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::table::Alignment;
use ic_host_text::timestamp::format_utc_timestamp_secs;

use crate::sns::common::dissolve_state_text;
use crate::sns::common::optional_e8s_text;
use crate::text_value::optional_bool_text;
use crate::text_value::optional_text;
use crate::text_value::optional_u64_text;
use crate::token_amount::e8s_decimal_text;
use ic_query::sns::SnsNeuronAccount;
use ic_query::sns::SnsNeuronDetail;
use ic_query::sns::SnsNeuronDetailReport;
use ic_query::sns::sns_neuron_permission_name;

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
    let (mint, staking) = detail.derived_policy_observations();
    let mut lines = crate::sns::common::sns_identity_lines(&report.context);
    lines.extend([
        field("neuron_id", &(report.neuron_id)),
        field("data_source", &(report.data_source)),
        field(
            "maturity_mint_conversion_observed_disabled",
            &(mint.as_str()),
        ),
        field(
            "manual_maturity_staking_observed_disabled",
            &(staking.as_str()),
        ),
        field(
            "cached_neuron_stake",
            &(e8s_decimal_text(neuron.cached_neuron_stake_e8s)),
        ),
        field("neuron_fees", &(e8s_decimal_text(neuron.neuron_fees_e8s))),
        field(
            "maturity",
            &(e8s_decimal_text(neuron.maturity_e8s_equivalent)),
        ),
        field(
            "staked_maturity",
            &(optional_e8s_text(neuron.staked_maturity_e8s_equivalent)),
        ),
        field(
            "auto_stake_maturity",
            &(optional_bool_text(neuron.auto_stake_maturity)),
        ),
        field(
            "dissolve_state",
            &(dissolve_state_text(neuron.dissolve_state)),
        ),
        field(
            "voting_power_percentage_multiplier",
            &(neuron.voting_power_percentage_multiplier),
        ),
        field(
            "created_at",
            &(format_utc_timestamp_secs(neuron.created_timestamp_seconds)),
        ),
        field(
            "source_nns_neuron_id",
            &(optional_u64_text(neuron.source_nns_neuron_id)),
        ),
        field(
            "vesting_period_seconds",
            &(optional_u64_text(neuron.vesting_period_seconds)),
        ),
        field("permission_entry_count", &(detail.permissions.len())),
        field(
            "pending_maturity_disbursement_count",
            &(detail.disburse_maturity_in_progress.len()),
        ),
        field("legacy_following_entry_count", &(detail.followees.len())),
        field(
            "topic_following_entry_count",
            &(detail.topic_followees.as_ref().map_or(0, Vec::len)),
        ),
    ]);
    lines.extend(crate::sns::common::sns_collection_lines(&report.context));
    lines
}

fn permission_table(detail: &SnsNeuronDetail) -> Option<String> {
    (!detail.permissions.is_empty()).then(|| {
        table(
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
                            .map(|value| format!("{value}:{}", sns_neuron_permission_name(*value)))
                            .collect::<Vec<_>>()
                            .join(","),
                    ]
                })
                .collect::<Vec<_>>(),
            &[Alignment::Left; 2],
        )
    })
}

fn maturity_disbursement_table(detail: &SnsNeuronDetail) -> Option<String> {
    (!detail.disburse_maturity_in_progress.is_empty()).then(|| {
        table(
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
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
                Alignment::Right,
            ],
        )
    })
}

fn legacy_followee_table(detail: &SnsNeuronDetail) -> Option<String> {
    (!detail.followees.is_empty()).then(|| {
        table(
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
            &[Alignment::Right, Alignment::Left],
        )
    })
}

fn topic_followee_table(detail: &SnsNeuronDetail) -> Option<String> {
    detail
        .topic_followees
        .as_ref()
        .filter(|topic_followees| !topic_followees.is_empty())
        .map(|topic_followees| {
            table(
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
                &[Alignment::Right, Alignment::Left, Alignment::Left],
            )
        })
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
