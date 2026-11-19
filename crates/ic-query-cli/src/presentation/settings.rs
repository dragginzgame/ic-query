//! Raw configured settings; effective defaults and capacity belong to other contracts.

use ic_host_text::bordered_table;
use ic_query_topology::subnet::settings::SubnetSettingsReport;

pub(crate) fn render(report: &SubnetSettingsReport) -> String {
    let rows = rows(&report.settings);
    let e = &report.evidence;

    format!(
        "Subnet: {}\n\nRegistered settings\n\n{}\n\nSource: {} / {} / {}\nKey: {}\nObserved: {} Unix seconds | record mutation version: {}\nRecord mutation timestamp: {} Unix nanoseconds\nQuery signature verified: {} | result consensus-certified: {} | TEE attestation verified: {}\n\nDefault-selected values remain unresolved; absent values are not zero or unlimited.\nConfigured features do not establish running behavior, usable TEE protection or reduced authentication requirements.\nThese declarations do not measure free capacity.\n",
        report.subnet_id,
        bordered_table(&["SETTING", "RAW VALUE / PRESENCE", "UNIT"], &rows),
        e.endpoint,
        e.source_canister,
        e.source_method,
        e.source_key,
        e.observed_at_unix_seconds,
        e.record_version,
        e.record_timestamp_nanoseconds,
        e.query_signature_verified,
        e.result_certified,
        e.tee_attestation_verified
    )
}

fn rows(settings: &ic_query_topology::subnet::settings::SubnetSettings) -> Vec<Vec<String>> {
    let limits = settings.resource_limits;
    let features = settings.features;
    let optional = |value: Option<u64>, default_selected: bool| {
        value.map_or_else(|| "absent".into(), |value| number(value, default_selected))
    };
    let rows = vec![
        vec![
            "Maximum canisters".into(),
            number(settings.max_number_of_canisters, true),
            "canisters".into(),
        ],
        vec![
            "Features message".into(),
            if features.is_some() {
                "present"
            } else {
                "absent"
            }
            .into(),
            "—".into(),
        ],
        vec![
            "HTTP requests configured".into(),
            features.map_or_else(|| "absent".into(), |f| f.http_requests.to_string()),
            "bool".into(),
        ],
        vec![
            "SEV configured".into(),
            features
                .and_then(|f| f.sev_enabled)
                .map_or_else(|| "absent".into(), |v| v.to_string()),
            "bool".into(),
        ],
        vec![
            "Resource limits message".into(),
            if limits.is_some() {
                "present"
            } else {
                "absent"
            }
            .into(),
            "—".into(),
        ],
        vec![
            "Maximum state size".into(),
            optional(limits.and_then(|l| l.maximum_state_size), false),
            "bytes".into(),
        ],
        vec![
            "Maximum state delta".into(),
            optional(limits.and_then(|l| l.maximum_state_delta), false),
            "bytes".into(),
        ],
        vec![
            "Maximum query instructions".into(),
            optional(limits.and_then(|l| l.maximum_query_instructions), true),
            "instructions".into(),
        ],
        vec![
            "Maximum query wall time".into(),
            optional(limits.and_then(|l| l.maximum_query_walltime_seconds), true),
            "seconds".into(),
        ],
    ];

    rows
}

fn number(value: u64, default_selected: bool) -> String {
    if value == 0 && default_selected {
        "0 (replica default selected)".into()
    } else {
        value.to_string()
    }
}

pub(crate) fn comparison(
    report: &ic_query_topology::subnet::compare::SettingsComparisonReport,
) -> String {
    use std::fmt::Write;
    let left = report.left_settings.report();
    let right = report.right_settings.report();
    let defaults = ic_query_topology::subnet::settings::SubnetSettings::default();
    let left_rows = rows(left.map_or(&defaults, |r| &r.settings));
    let right_rows = rows(right.map_or(&defaults, |r| &r.settings));
    let combined: Vec<_> = left_rows
        .iter()
        .zip(&right_rows)
        .map(|(a, b)| {
            vec![
                a[0].clone(),
                if left.is_some() {
                    a[1].clone()
                } else {
                    "unavailable".into()
                },
                if right.is_some() {
                    b[1].clone()
                } else {
                    "unavailable".into()
                },
                a[2].clone(),
            ]
        })
        .collect();
    let mut text = format!(
        "\nRegistered settings (independent reads)\n\n{}\n",
        bordered_table(&["SETTING", "LEFT", "RIGHT", "UNIT"], &combined)
    );
    for (label, observation) in [("Left", left), ("Right", right)] {
        if let Some(r) = observation {
            let e = &r.evidence;
            writeln!(text, "{label} settings: {} / {} / {}\nKey: {}\nObserved: {} Unix seconds | record mutation version: {} | mutation time: {} Unix nanoseconds\nSignature verified: {} | consensus-certified: {} | attestation verified: {}", e.endpoint, e.source_canister, e.source_method, e.source_key, e.observed_at_unix_seconds, e.record_version, e.record_timestamp_nanoseconds, e.query_signature_verified, e.result_certified, e.tee_attestation_verified).expect("String write");
        }
    }
    text.push_str("Settings and distribution versions are independent. Absent declarations are not zero or unlimited; replica defaults remain unresolved.\nConfigured SEV does not establish usable TEE protection or reduced authentication requirements. No free-capacity estimate is derived.\n");

    text
}
