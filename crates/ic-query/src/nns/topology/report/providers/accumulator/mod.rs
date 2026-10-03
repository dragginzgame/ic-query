mod ingest;
mod rows;

use crate::nns::data_center::NnsDataCenterListReport;
use std::collections::{BTreeMap, BTreeSet};

///
/// NnsTopologyProviderAccumulator
///
/// Mutable join state used to aggregate topology rows by node provider.
///

pub(super) struct NnsTopologyProviderAccumulator {
    data_center_regions: BTreeMap<String, String>,
    providers: BTreeMap<String, ProviderAggregate>,
}

///
/// ProviderAggregate
///
/// Registration evidence, observed counts, and distinct locations for one provider.
///

#[derive(Default)]
struct ProviderAggregate {
    metadata: Option<(Option<String>, Option<u64>)>,
    topology_node_count: u64,
    node_operator_count: u64,
    data_center_ids: BTreeSet<String>,
    region_ids: BTreeSet<String>,
    node_allowance: u64,
    assigned_node_count: u64,
    available_node_slots: u64,
    over_assigned_node_count: u64,
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
            providers: BTreeMap::new(),
        }
    }
}
