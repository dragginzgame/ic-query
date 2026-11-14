//! Pure human-facing projections; process output remains in the entrypoint.

use ic_topology::placement::PlacementReport;
use ic_topology::subnet::SubnetReport;
use ic_topology::subnet::list::SubnetListReport;

pub(crate) fn subnet_list(report: &SubnetListReport) -> String {
    let subnets = report
        .subnet_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Source: {} / {} / {} / {}\nObserved at: {} Unix seconds\nRecord mutation version: {}\nRecord mutation timestamp: {} Unix nanoseconds\nQuery signature verified: {}\nResult consensus-certified: {}\nSelected record complete: {}\n\nRegistered subnets ({}):\n{}",
        report.evidence.endpoint,
        report.evidence.source_canister,
        report.evidence.source_method,
        report.evidence.source_key,
        report.evidence.observed_at_unix_seconds,
        report.evidence.record_version,
        report.evidence.record_timestamp_nanoseconds,
        report.evidence.query_signature_verified,
        report.evidence.result_certified,
        report.evidence.selected_record_complete,
        report.subnet_ids.len(),
        subnets,
    )
}

pub(crate) fn placement(report: &PlacementReport) -> String {
    format!(
        "Source: {} / {} / {}\nObserved at: {} Unix seconds\nQuery signature verified: {}\nResult consensus-certified: {}\nSelected lookup complete: {}\nRegistry version: {}\n\nCanister: {}\nSubnet: {}",
        report.evidence.endpoint,
        report.evidence.source_canister,
        report.evidence.source_method,
        report.evidence.observed_at_unix_seconds,
        report.evidence.query_signature_verified,
        report.evidence.result_certified,
        report.evidence.selected_lookup_complete,
        registry_version(report.evidence.registry_version),
        report.canister_id,
        report.subnet_id,
    )
}

pub(crate) fn subnet(report: &SubnetReport) -> String {
    let nodes = report
        .observed_node_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Source: {}{}\nObserved at: {} Unix seconds\nState certificate verified: {}\nComplete Registry membership established: {}\nRegistry version: {}\n\nSubnet: {}\nObserved node identities ({}):\n{}",
        report.evidence.endpoint,
        report.evidence.source_path,
        report.evidence.observed_at_unix_seconds,
        report.evidence.state_certificate_verified,
        report.evidence.membership_complete,
        registry_version(report.evidence.registry_version),
        report.subnet_id,
        report.observed_node_ids.len(),
        nodes,
    )
}

fn registry_version(version: Option<u64>) -> String {
    version.map_or_else(|| "unavailable".to_owned(), |value| value.to_string())
}

#[cfg(test)]
mod tests;
