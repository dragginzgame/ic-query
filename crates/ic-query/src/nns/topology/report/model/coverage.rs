use serde::{Deserialize, Serialize};

///
/// NnsTopologyCoverageReport
///
/// NNS topology report describing registry relation join coverage.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCoverageReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub node_count: usize,
    pub node_provider_count: usize,
    pub node_operator_count: usize,
    pub data_center_count: usize,
    /// Registry relation coverage, serialized as flat report counts.
    #[serde(flatten)]
    pub join_coverage: NnsTopologyJoinCoverageCounts,
}

///
/// NnsTopologyJoinCoverageCounts
///
/// Relation-coverage counts shared by coverage and summary reports.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyJoinCoverageCounts {
    /// Number of nodes with known node provider references.
    pub nodes_with_known_node_provider_count: usize,
    /// Number of nodes with unknown node provider references.
    pub nodes_with_unknown_node_provider_count: usize,
    /// Number of nodes with known node operator references.
    pub nodes_with_known_node_operator_count: usize,
    /// Number of nodes with unknown node operator references.
    pub nodes_with_unknown_node_operator_count: usize,
    /// Number of nodes with known data center references.
    pub nodes_with_known_data_center_count: usize,
    /// Number of nodes with unknown data center references.
    pub nodes_with_unknown_data_center_count: usize,
    /// Number of node operators with known node provider references.
    pub node_operators_with_known_node_provider_count: usize,
    /// Number of node operators with unknown node provider references.
    pub node_operators_with_unknown_node_provider_count: usize,
    /// Number of node operators with known data center references.
    pub node_operators_with_known_data_center_count: usize,
    /// Number of node operators with unknown data center references.
    pub node_operators_with_unknown_data_center_count: usize,
}

impl NnsTopologyJoinCoverageCounts {
    pub(in crate::nns::topology::report) const fn join_counts(&self) -> [(usize, usize); 5] {
        [
            (
                self.nodes_with_known_node_provider_count,
                self.nodes_with_unknown_node_provider_count,
            ),
            (
                self.nodes_with_known_node_operator_count,
                self.nodes_with_unknown_node_operator_count,
            ),
            (
                self.nodes_with_known_data_center_count,
                self.nodes_with_unknown_data_center_count,
            ),
            (
                self.node_operators_with_known_node_provider_count,
                self.node_operators_with_unknown_node_provider_count,
            ),
            (
                self.node_operators_with_known_data_center_count,
                self.node_operators_with_unknown_data_center_count,
            ),
        ]
    }
}
