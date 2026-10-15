//! Module: sns::report::text::upgrade
//!
//! Responsibility: render bounded SNS upgrade reports as human-facing text.
//! Does not own: live calls, report construction, source validation, or JSON output.
//! Boundary: labels Governance and SNS-W evidence precisely and preserves query gaps.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use crate::{
    sns::report::{SnsPendingUpgrade, SnsUpgradeReport, SnsVersion},
    text_value::yes_no,
};

/// Render one SNS upgrade report as human-facing text.
#[must_use]
pub fn sns_upgrade_report_text(report: &SnsUpgradeReport) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", terminal_text(&report.name)),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("component_query_count: {}", report.component_query_count),
        format!(
            "successful_component_query_count: {}",
            report.successful_component_query_count
        ),
        format!("component_gap_count: {}", report.component_gap_count),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!(
            "running_version_method: {}",
            report.running_version_method.as_str()
        ),
        format!(
            "next_version_method: {}",
            report.next_version_method.as_str()
        ),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
        String::new(),
        next_version_status(report),
        String::new(),
        "versions:".to_string(),
        comparison_table(&report.deployed_version, report.next_version.as_ref()),
        String::new(),
        "pending_upgrade:".to_string(),
        pending_upgrade_text(report.pending_upgrade.as_ref()),
    ];
    if let Some(gap) = &report.next_version_gap {
        lines.extend([
            String::new(),
            "next_version_gap:".to_string(),
            render(
                &["METHOD", "REASON"],
                &[[gap.method.as_str().to_string(), terminal_text(&gap.reason)]],
                &[Alignment::Left, Alignment::Left],
                REPORT_TABLE_OPTIONS,
            ),
        ]);
    }
    lines.join("\n")
}

fn next_version_status(report: &SnsUpgradeReport) -> String {
    if report.next_version.is_some() {
        "next_version: available".to_string()
    } else if report.next_version_gap.is_some() {
        "next_version: unavailable (query failed)".to_string()
    } else {
        "next_version: none (no blessed successor)".to_string()
    }
}

fn comparison_table(deployed: &SnsVersion, next: Option<&SnsVersion>) -> String {
    let deployed_hashes = version_hashes(deployed);
    let next_hashes = next.map(version_hashes);
    let rows = deployed_hashes
        .iter()
        .enumerate()
        .map(|(index, (role, deployed_hash))| {
            let next_hash = next_hashes.as_ref().map_or("-", |hashes| hashes[index].1);
            let changed = next_hashes.as_ref().map_or("-", |hashes| {
                if deployed_hash == &hashes[index].1 {
                    "no"
                } else {
                    "yes"
                }
            });
            [
                (*role).to_string(),
                (*deployed_hash).to_string(),
                next_hash.to_string(),
                changed.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    render(
        &["ROLE", "DEPLOYED", "NEXT", "CHANGED"],
        &rows,
        &[
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
        ],
        REPORT_TABLE_OPTIONS,
    )
}

fn pending_upgrade_text(pending: Option<&SnsPendingUpgrade>) -> String {
    let Some(pending) = pending else {
        return "-".to_string();
    };
    let mut lines = vec![render(
        &["FIELD", "VALUE"],
        &[
            [
                "mark_failed_at_seconds".to_string(),
                pending.mark_failed_at_seconds.to_string(),
            ],
            [
                "checking_upgrade_lock".to_string(),
                pending.checking_upgrade_lock.to_string(),
            ],
            ["proposal_id".to_string(), pending.proposal_id.to_string()],
        ],
        &[Alignment::Left, Alignment::Right],
        REPORT_TABLE_OPTIONS,
    )];
    if let Some(target) = &pending.target_version {
        lines.extend([
            String::new(),
            "target_version:".to_string(),
            single_version_table(target),
        ]);
    } else {
        lines.push("target_version: -".to_string());
    }
    lines.join("\n")
}

fn single_version_table(version: &SnsVersion) -> String {
    render(
        &["ROLE", "HASH"],
        &version_hashes(version)
            .into_iter()
            .map(|(role, hash)| [role.to_string(), hash.to_string()])
            .collect::<Vec<_>>(),
        &[Alignment::Left, Alignment::Left],
        REPORT_TABLE_OPTIONS,
    )
}

fn version_hashes(version: &SnsVersion) -> [(&'static str, &str); 6] {
    [
        ("archive", &version.archive_wasm_hash_hex),
        ("root", &version.root_wasm_hash_hex),
        ("swap", &version.swap_wasm_hash_hex),
        ("ledger", &version.ledger_wasm_hash_hex),
        ("governance", &version.governance_wasm_hash_hex),
        ("index", &version.index_wasm_hash_hex),
    ]
}
