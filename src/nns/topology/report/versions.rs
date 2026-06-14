use super::{
    NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION, NnsTopologySummaryReport,
    NnsTopologyVersionsReport,
};

pub(super) fn topology_versions_report_from_summary(
    summary: NnsTopologySummaryReport,
) -> NnsTopologyVersionsReport {
    NnsTopologyVersionsReport {
        schema_version: NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION,
        network: summary.network,
        source_endpoint: summary.source_endpoint,
        source_count: summary.registry_versions.len(),
        registry_versions: summary.registry_versions,
    }
}
