mod ingest;
mod rows;

use crate::nns::data_center::report::NnsDataCenterListReport;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct NnsTopologyProviderAccumulator {
    data_center_regions: BTreeMap<String, String>,
    provider_principals: BTreeSet<String>,
    provider_metadata: BTreeMap<String, (Option<String>, Option<u64>)>,
    topology_node_counts: BTreeMap<String, u64>,
    node_operator_counts: BTreeMap<String, u64>,
    data_center_ids: BTreeMap<String, BTreeSet<String>>,
    region_ids: BTreeMap<String, BTreeSet<String>>,
    node_allowances: BTreeMap<String, u64>,
    assigned_node_counts: BTreeMap<String, u64>,
    available_node_slots: BTreeMap<String, u64>,
    over_assigned_node_counts: BTreeMap<String, u64>,
}

impl NnsTopologyProviderAccumulator {
    pub(super) fn from_data_centers(report: &NnsDataCenterListReport) -> Self {
        Self {
            data_center_regions: report
                .data_centers
                .iter()
                .map(|data_center| {
                    (
                        data_center.data_center_id.clone(),
                        data_center.region.clone(),
                    )
                })
                .collect(),
            provider_principals: BTreeSet::new(),
            provider_metadata: BTreeMap::new(),
            topology_node_counts: BTreeMap::new(),
            node_operator_counts: BTreeMap::new(),
            data_center_ids: BTreeMap::new(),
            region_ids: BTreeMap::new(),
            node_allowances: BTreeMap::new(),
            assigned_node_counts: BTreeMap::new(),
            available_node_slots: BTreeMap::new(),
            over_assigned_node_counts: BTreeMap::new(),
        }
    }
}
