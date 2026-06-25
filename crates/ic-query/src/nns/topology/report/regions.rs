use super::{
    NNS_TOPOLOGY_REGIONS_REPORT_SCHEMA_VERSION, NnsTopologyRegionRow, NnsTopologyRegionsReport,
};
use crate::nns::data_center::report::NnsDataCenterListReport;
use std::collections::BTreeMap;

pub(super) fn topology_regions_report_from_report(
    network: String,
    source_endpoint: String,
    data_center_report: NnsDataCenterListReport,
) -> NnsTopologyRegionsReport {
    let mut region_map = BTreeMap::<String, NnsTopologyRegionRow>::new();
    for data_center in &data_center_report.data_centers {
        let row = region_map
            .entry(data_center.region.clone())
            .or_insert_with(|| NnsTopologyRegionRow {
                region: data_center.region.clone(),
                data_center_count: 0,
                node_operator_count: 0,
                node_provider_count: 0,
                node_count: 0,
            });
        row.data_center_count = row.data_center_count.saturating_add(1);
        row.node_operator_count = row
            .node_operator_count
            .saturating_add(u64::from(data_center.node_operator_count));
        row.node_provider_count = row
            .node_provider_count
            .saturating_add(u64::from(data_center.node_provider_count));
        row.node_count = row
            .node_count
            .saturating_add(u64::from(data_center.node_count));
    }

    let mut regions = region_map.into_values().collect::<Vec<_>>();
    regions.sort_by(|left, right| {
        (
            std::cmp::Reverse(left.node_count),
            std::cmp::Reverse(left.data_center_count),
            left.region.as_str(),
        )
            .cmp(&(
                std::cmp::Reverse(right.node_count),
                std::cmp::Reverse(right.data_center_count),
                right.region.as_str(),
            ))
    });
    let node_operator_count = regions.iter().map(|row| row.node_operator_count).sum();
    let node_provider_count = regions.iter().map(|row| row.node_provider_count).sum();
    let node_count = regions.iter().map(|row| row.node_count).sum();

    NnsTopologyRegionsReport {
        schema_version: NNS_TOPOLOGY_REGIONS_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        region_count: regions.len(),
        data_center_count: data_center_report.data_center_count,
        node_operator_count,
        node_provider_count,
        node_count,
        regions,
    }
}
