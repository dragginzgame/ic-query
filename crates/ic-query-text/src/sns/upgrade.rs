//! Module: sns::upgrade
//!
//! Responsibility: render bounded SNS upgrade reports as human-facing text.
//! Does not own: live calls, report construction, source validation, or JSON output.
//! Boundary: labels Governance and SNS-W evidence precisely and preserves query gaps.

use crate::sns::common::sns_identity_lines;
use crate::text_value::field;
use crate::text_value::table;

use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::text_value::yes_no;
use ic_query::sns::SnsPendingUpgrade;
use ic_query::sns::SnsUpgradeReport;
use ic_query::sns::SnsVersion;

/// Render one SNS upgrade report as human-facing text.
#[must_use]
pub fn sns_upgrade_report_text(report: &SnsUpgradeReport) -> String {
    let mut lines = sns_identity_lines(&report.context);
    lines.extend([
        field("component_query_count", &(report.component_query_count)),
        field(
            "successful_component_query_count",
            &(report.successful_component_query_count),
        ),
        field("component_gap_count", &(report.component_gap_count)),
        field(
            "point_in_time_guaranteed",
            &(yes_no(report.point_in_time_guaranteed)),
        ),
        field(
            "running_version_method",
            &(report.running_version_method.as_str()),
        ),
        field(
            "next_version_method",
            &(report.next_version_method.as_str()),
        ),
        field(
            "sns_wasm_canister_id",
            &(report.context.instance.sns_wasm_canister_id),
        ),
        field("fetched_at", &report.context.instance.fetched_at),
        field("source_endpoint", &report.context.instance.source_endpoint),
        String::new(),
        next_version_status(report),
        String::new(),
        "versions:".to_string(),
        comparison_table(&report.deployed_version, report.next_version.as_ref()),
        String::new(),
        "pending_upgrade:".to_string(),
        pending_upgrade_text(report.pending_upgrade.as_ref()),
    ]);
    if let Some(gap) = &report.next_version_gap {
        lines.extend([
            String::new(),
            "next_version_gap:".to_string(),
            table(
                &["METHOD", "REASON"],
                &[[
                    gap.method.as_str().to_string(),
                    terminal_text(&gap.reason).into_owned(),
                ]],
                &[Alignment::Left; 2],
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
    table(
        &["ROLE", "DEPLOYED", "NEXT", "CHANGED"],
        &rows,
        &[Alignment::Left; 4],
    )
}

fn pending_upgrade_text(pending: Option<&SnsPendingUpgrade>) -> String {
    let Some(pending) = pending else {
        return "-".to_string();
    };
    let mut lines = vec![table(
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
    table(
        &["ROLE", "HASH"],
        &version_hashes(version)
            .into_iter()
            .map(|(role, hash)| [role.to_string(), hash.to_string()])
            .collect::<Vec<_>>(),
        &[Alignment::Left; 2],
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
