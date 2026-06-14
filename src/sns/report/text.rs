use super::{
    SnsGovernanceParameters, SnsInfoReport, SnsListReport, SnsNeuronPermissionList,
    SnsNeuronsCacheListReport, SnsNeuronsCacheStatusReport, SnsNeuronsRefreshAttemptStatus,
    SnsNeuronsRefreshReport, SnsNeuronsReport, SnsParamsReport, SnsProposalReport, SnsProposalRow,
    SnsProposalsReport, SnsTokenMetadataRow, SnsTokenReport, short_principal,
};
use crate::{
    duration::display_duration_seconds,
    nns::render::yes_no,
    table::{ColumnAlign, render_table},
    token_amount::{base_units_decimal_text, e8s_decimal_text},
};
use serde_json::Value as JsonValue;

const COMPACT_NEURON_ID_CHARS: usize = 8;
const SNS_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;
const SNS_PROPOSAL_TITLE_TEXT_LIMIT: usize = 96;
const SNS_PROPOSAL_DETAIL_TEXT_LIMIT: usize = 240;

#[must_use]
pub fn sns_list_report_text(report: &SnsListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("network: {}", report.network));
    lines.push(format!(
        "sns_wasm_canister_id: {}",
        report.sns_wasm_canister_id
    ));
    lines.push(format!("sns_count: {}", report.sns_count));
    lines.push(format!("fetched_at: {}", report.fetched_at));
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("sort: {}", report.sort));
    lines.push(format!("metadata_errors: {}", report.metadata_error_count));
    if !report.sns_instances.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "ID",
                "NAME",
                "ROOT",
                "GOVERNANCE",
                "LEDGER",
                "SWAP",
                "INDEX",
            ],
            &report
                .sns_instances
                .iter()
                .map(|sns| {
                    [
                        sns.id.to_string(),
                        sns.name.clone(),
                        principal_for_list(&sns.root_canister_id, report.verbose),
                        principal_for_list(&sns.governance_canister_id, report.verbose),
                        principal_for_list(&sns.ledger_canister_id, report.verbose),
                        principal_for_list(&sns.swap_canister_id, report.verbose),
                        principal_for_list(&sns.index_canister_id, report.verbose),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }
    if report.verbose && report.metadata_error_count > 0 {
        lines.push(String::new());
        lines.push("metadata_error_details:".to_string());
        for (governance_canister_id, error) in report.sns_instances.iter().filter_map(|sns| {
            sns.metadata_error
                .as_deref()
                .map(|error| (&sns.governance_canister_id, error))
        }) {
            lines.push(format!("- {governance_canister_id}: {error}"));
        }
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_info_report_text(report: &SnsInfoReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!(
            "description: {}",
            optional_text(report.description.as_ref())
        ),
        format!("url: {}", optional_text(report.url.as_ref())),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("swap_canister_id: {}", report.swap_canister_id),
        format!("index_canister_id: {}", report.index_canister_id),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.metadata_error.as_deref() {
        lines.push(format!("metadata_error: {error}"));
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_token_report_text(report: &SnsTokenReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("sns_index_canister_id: {}", report.sns_index_canister_id),
        format!(
            "ledger_index_canister_id: {}",
            optional_text(report.ledger_index_canister_id.as_ref())
        ),
        format!("token_name: {}", report.token_name),
        format!("token_symbol: {}", report.token_symbol),
        format!("decimals: {}", report.decimals),
        format!(
            "transfer_fee: {}",
            base_units_decimal_text(&report.transfer_fee, report.decimals)
        ),
        format!(
            "total_supply: {}",
            base_units_decimal_text(&report.total_supply, report.decimals)
        ),
        format!(
            "minting_account_owner: {}",
            optional_text(report.minting_account_owner.as_ref())
        ),
        format!(
            "minting_account_subaccount_hex: {}",
            optional_text(report.minting_account_subaccount_hex.as_ref())
        ),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.ledger_index_error.as_deref() {
        lines.push(format!("ledger_index_error: {error}"));
    }
    if !report.supported_standards.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["STANDARD", "URL"],
            &report
                .supported_standards
                .iter()
                .map(|standard| [standard.name.clone(), standard.url.clone()])
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    if !report.metadata.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["METADATA", "TYPE", "VALUE"],
            &report
                .metadata
                .iter()
                .map(|row| {
                    [
                        row.key.clone(),
                        row.value_type.clone(),
                        truncate_text_value(
                            &token_metadata_value_text(row, report.decimals),
                            SNS_TOKEN_METADATA_TEXT_VALUE_LIMIT,
                        ),
                    ]
                })
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_params_report_text(report: &SnsParamsReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    lines.push(String::new());
    lines.push(render_table(
        &["PARAMETER", "VALUE"],
        &sns_params_text_rows(&report.parameters),
        &[ColumnAlign::Left, ColumnAlign::Right],
    ));
    lines.join("\n")
}

#[must_use]
pub fn sns_proposal_report_text(report: &SnsProposalReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("proposal_id: {}", report.proposal_id),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        String::new(),
        "proposal:".to_string(),
    ];
    let detail_limit = (!report.verbose).then_some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT);
    lines.extend(sns_proposal_detail_lines(&report.proposal, detail_limit));
    lines.join("\n")
}

#[must_use]
pub fn sns_proposals_report_text(report: &SnsProposalsReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "before_proposal_id: {}",
            optional_u64_text(report.before_proposal_id)
        ),
        format!("status_filter: {}", report.status_filter),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("proposal_count: {}", report.proposal_count),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["ID", "ACTION", "DECISION", "CREATED_AT", "TITLE"],
            &report
                .proposals
                .iter()
                .map(|proposal| {
                    [
                        optional_u64_text(proposal.proposal_id),
                        proposal.action.clone(),
                        proposal.decision_state.clone(),
                        proposal.created_at.clone(),
                        proposal_title_for_list(proposal, report.verbose),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }
    if report.verbose && !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push("proposal_details:".to_string());
        for proposal in &report.proposals {
            lines.extend(sns_proposal_detail_lines(
                proposal,
                Some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT),
            ));
        }
    }
    lines.join("\n")
}

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
        format!("data_source: {}", report.data_source),
        format!("sort: {}", report.sort),
        format!("total_neuron_count: {}", report.total_neuron_count),
        format!("neuron_count: {}", report.neuron_count),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(cache_path) = report.cache_path.as_deref() {
        lines.push(format!("cache_path: {cache_path}"));
    }
    if let Some(cache_complete) = report.cache_complete {
        lines.push(format!("cache_complete: {}", yes_no(cache_complete)));
    }
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

#[must_use]
pub fn sns_neurons_refresh_report_text(report: &SnsNeuronsRefreshReport) -> String {
    [
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("neuron_count: {}", report.neuron_count),
        format!("complete: {}", yes_no(report.complete)),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("cache_path: {}", report.cache_path),
        format!("refresh_lock_path: {}", report.refresh_lock_path),
        format!("refresh_attempt_path: {}", report.refresh_attempt_path),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]
    .join("\n")
}

#[must_use]
pub fn sns_neurons_cache_list_report_text(report: &SnsNeuronsCacheListReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("cache_root: {}", report.cache_root),
        format!("cache_count: {}", report.cache_count),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "ID",
                "NAME",
                "ROOT",
                "COMPLETE",
                "ROWS",
                "PAGES",
                "FETCHED_AT",
            ],
            &report
                .caches
                .iter()
                .map(|cache| {
                    [
                        cache.id.to_string(),
                        cache.name.clone(),
                        short_principal(&cache.root_canister_id),
                        yes_no(cache.complete).to_string(),
                        cache.row_count.to_string(),
                        cache.page_count.to_string(),
                        cache.fetched_at.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_neurons_cache_status_report_text(report: &SnsNeuronsCacheStatusReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("input: {}", report.input),
        format!("cache_root: {}", report.cache_root),
        format!("found: {}", yes_no(report.found)),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("sns_id: {}", cache.id),
            format!("name: {}", cache.name),
            format!("root_canister_id: {}", cache.root_canister_id),
            format!("governance_canister_id: {}", cache.governance_canister_id),
            format!("complete: {}", yes_no(cache.complete)),
            format!("row_count: {}", cache.row_count),
            format!("page_count: {}", cache.page_count),
            format!("page_size: {}", cache.page_size),
            format!("fetched_at: {}", cache.fetched_at),
            format!("source_endpoint: {}", cache.source_endpoint),
            format!("cache_path: {}", cache.cache_path),
            format!("refresh_attempt_path: {}", cache.refresh_attempt_path),
        ]);
    } else {
        if let Some(cache_path) = report.expected_cache_path.as_deref() {
            lines.push(format!("expected_cache_path: {cache_path}"));
        }
        if let Some(attempt_path) = report.refresh_attempt_path.as_deref() {
            lines.push(format!("refresh_attempt_path: {attempt_path}"));
        }
        lines.push(format!(
            "refresh_hint: icq sns neurons refresh {}",
            report.input
        ));
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.push(String::new());
        lines.extend(sns_neurons_attempt_text_rows(attempt));
    }
    lines.join("\n")
}

fn sns_neurons_attempt_text_rows(attempt: &SnsNeuronsRefreshAttemptStatus) -> [String; 8] {
    [
        format!("latest_attempt_status: {}", attempt.status),
        format!("latest_attempt_started_at: {}", attempt.started_at),
        format!("latest_attempt_updated_at: {}", attempt.updated_at),
        format!("latest_attempt_page_size: {}", attempt.page_size),
        format!("latest_attempt_pages_fetched: {}", attempt.pages_fetched),
        format!("latest_attempt_rows_fetched: {}", attempt.rows_fetched),
        format!(
            "latest_attempt_last_cursor: {}",
            optional_text(attempt.last_cursor.as_ref())
        ),
        format!(
            "latest_attempt_last_error: {}",
            optional_text(attempt.last_error.as_ref())
        ),
    ]
}

fn principal_for_list(value: &str, verbose: bool) -> String {
    if verbose {
        value.to_string()
    } else {
        short_principal(value)
    }
}

fn neuron_id_for_list(value: &str, verbose: bool) -> String {
    if verbose || value == "-" {
        value.to_string()
    } else {
        value.chars().take(COMPACT_NEURON_ID_CHARS).collect()
    }
}

fn optional_text(value: Option<&String>) -> &str {
    value.map_or("-", String::as_str)
}

fn proposal_title_for_list(proposal: &SnsProposalRow, verbose: bool) -> String {
    if verbose {
        proposal.title.clone()
    } else {
        truncate_text_value(&proposal.title, SNS_PROPOSAL_TITLE_TEXT_LIMIT)
    }
}

fn sns_proposal_detail_lines(
    proposal: &SnsProposalRow,
    detail_limit: Option<usize>,
) -> Vec<String> {
    let mut lines = vec![
        format!("- proposal_id: {}", optional_u64_text(proposal.proposal_id)),
        format!("  action_id: {}", proposal.action_id),
        format!("  action: {}", proposal.action),
        format!("  decision_state: {}", proposal.decision_state),
        format!("  title: {}", proposal.title),
        format!("  url: {}", optional_text(proposal.url.as_ref())),
        format!(
            "  proposer_neuron_id: {}",
            optional_text(proposal.proposer_neuron_id.as_ref())
        ),
        format!(
            "  reject_cost: {}",
            e8s_decimal_text(proposal.reject_cost_e8s)
        ),
        format!("  created_at: {}", proposal.created_at),
        format!(
            "  decided_at: {}",
            optional_text(proposal.decided_at.as_ref())
        ),
        format!(
            "  executed_at: {}",
            optional_text(proposal.executed_at.as_ref())
        ),
        format!(
            "  failed_at: {}",
            optional_text(proposal.failed_at.as_ref())
        ),
        format!("  reward_event_round: {}", proposal.reward_event_round),
        format!(
            "  is_eligible_for_rewards: {}",
            yes_no(proposal.is_eligible_for_rewards)
        ),
        format!("  ballot_count: {}", proposal.ballot_count),
    ];
    if let Some(tally) = proposal.latest_tally.as_ref() {
        lines.extend([
            format!("  tally_yes: {}", tally.yes),
            format!("  tally_no: {}", tally.no),
            format!("  tally_total: {}", tally.total),
        ]);
    }
    if let Some(reason) = proposal.failure_reason.as_ref() {
        lines.extend([
            format!("  failure_error_type: {}", reason.error_type),
            format!("  failure_error_message: {}", reason.error_message),
        ]);
    }
    if !proposal.summary.is_empty() {
        lines.push(format!(
            "  summary: {}",
            proposal_detail_text(&proposal.summary, detail_limit)
        ));
    }
    if let Some(rendering) = proposal.payload_text_rendering.as_ref() {
        lines.push(format!(
            "  payload_text_rendering: {}",
            proposal_detail_text(rendering, detail_limit)
        ));
    }
    lines
}

fn proposal_detail_text(value: &str, detail_limit: Option<usize>) -> String {
    detail_limit.map_or_else(
        || value.to_string(),
        |limit| truncate_text_value(value, limit),
    )
}

fn sns_params_text_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    [
        sns_params_economic_rows(parameters),
        sns_params_delay_rows(parameters),
        sns_params_limit_rows(parameters),
        sns_params_permission_rows(parameters),
        sns_params_reward_rows(parameters),
    ]
    .concat()
}

fn sns_params_economic_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "neuron_minimum_stake",
            optional_e8s_text(parameters.neuron_minimum_stake_e8s),
        ),
        parameter_row(
            "transaction_fee",
            optional_e8s_text(parameters.transaction_fee_e8s),
        ),
        parameter_row("reject_cost", optional_e8s_text(parameters.reject_cost_e8s)),
    ]
}

fn sns_params_delay_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "max_dissolve_delay",
            optional_duration_text(parameters.max_dissolve_delay_seconds),
        ),
        parameter_row(
            "max_dissolve_delay_bonus",
            optional_percentage_text(parameters.max_dissolve_delay_bonus_percentage),
        ),
        parameter_row(
            "max_neuron_age_for_age_bonus",
            optional_duration_text(parameters.max_neuron_age_for_age_bonus),
        ),
        parameter_row(
            "max_age_bonus",
            optional_percentage_text(parameters.max_age_bonus_percentage),
        ),
        parameter_row(
            "initial_voting_period",
            optional_duration_text(parameters.initial_voting_period_seconds),
        ),
        parameter_row(
            "wait_for_quiet_deadline_increase",
            optional_duration_text(parameters.wait_for_quiet_deadline_increase_seconds),
        ),
        parameter_row(
            "minimum_dissolve_delay_to_vote",
            optional_duration_text(parameters.neuron_minimum_dissolve_delay_to_vote_seconds),
        ),
    ]
}

fn sns_params_limit_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "max_followees_per_function",
            optional_u64_text(parameters.max_followees_per_function),
        ),
        parameter_row(
            "max_proposals_to_keep_per_action",
            optional_u32_text(parameters.max_proposals_to_keep_per_action),
        ),
        parameter_row(
            "max_number_of_neurons",
            optional_u64_text(parameters.max_number_of_neurons),
        ),
        parameter_row(
            "max_number_of_proposals_with_ballots",
            optional_u64_text(parameters.max_number_of_proposals_with_ballots),
        ),
        parameter_row(
            "max_number_of_principals_per_neuron",
            optional_u64_text(parameters.max_number_of_principals_per_neuron),
        ),
        parameter_row(
            "maturity_modulation_disabled",
            optional_bool_text(parameters.maturity_modulation_disabled),
        ),
        parameter_row(
            "automatically_advance_target_version",
            optional_bool_text(parameters.automatically_advance_target_version),
        ),
    ]
}

fn sns_params_permission_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "neuron_claimer_permissions",
            optional_permissions_text(parameters.neuron_claimer_permissions.as_ref()),
        ),
        parameter_row(
            "neuron_grantable_permissions",
            optional_permissions_text(parameters.neuron_grantable_permissions.as_ref()),
        ),
    ]
}

fn sns_params_reward_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    let rewards = parameters.voting_rewards_parameters.as_ref();
    vec![
        parameter_row(
            "voting_reward_initial_rate",
            optional_basis_points_text(
                rewards.and_then(|rewards| rewards.initial_reward_rate_basis_points),
            ),
        ),
        parameter_row(
            "voting_reward_final_rate",
            optional_basis_points_text(
                rewards.and_then(|rewards| rewards.final_reward_rate_basis_points),
            ),
        ),
        parameter_row(
            "voting_reward_transition_duration",
            optional_duration_text(
                rewards.and_then(|rewards| rewards.reward_rate_transition_duration_seconds),
            ),
        ),
        parameter_row(
            "voting_reward_round_duration",
            optional_duration_text(rewards.and_then(|rewards| rewards.round_duration_seconds)),
        ),
        parameter_row(
            "additional_critical_native_actions",
            parameters.custom_proposal_criticality.as_ref().map_or_else(
                || "-".to_string(),
                |criticality| comma_join_u64(&criticality.additional_critical_native_action_ids),
            ),
        ),
    ]
}

fn parameter_row(name: &str, value: String) -> [String; 2] {
    [name.to_string(), value]
}

fn optional_e8s_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), e8s_decimal_text)
}

pub(super) fn optional_e8s_decimal_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), e8s_decimal_text)
}

fn optional_duration_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), display_duration_seconds)
}

fn optional_percentage_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| format!("{value}%"))
}

fn optional_basis_points_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), basis_points_text)
}

fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_u32_text(value: Option<u32>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_bool_text(value: Option<bool>) -> String {
    value.map_or_else(|| "-".to_string(), |value| yes_no(value).to_string())
}

fn optional_permissions_text(value: Option<&SnsNeuronPermissionList>) -> String {
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

fn comma_join_u64(values: &[u64]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn basis_points_text(value: u64) -> String {
    let whole = value / 100;
    let fractional = value % 100;
    format!("{whole}.{fractional:02}%")
}

fn truncate_text_value(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn metadata_value_text(value: &JsonValue) -> String {
    match value {
        JsonValue::String(value) => value.clone(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::Null => "-".to_string(),
        JsonValue::Array(_) | JsonValue::Object(_) => value.to_string(),
    }
}

fn token_metadata_value_text(row: &SnsTokenMetadataRow, decimals: u8) -> String {
    let value = metadata_value_text(&row.value);
    if row.key == "icrc1:fee" {
        base_units_decimal_text(&value, decimals)
    } else {
        value
    }
}
