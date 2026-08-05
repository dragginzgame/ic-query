//! Module: ic::source::node_status
//!
//! Responsibility: finite Dashboard node-status source contract and snapshot validation.
//! Does not own: HTTP transport, cache policy, aggregate projections, or rendering.
//! Boundary: converts one untrusted source response into a canonical observed snapshot.

use super::{invalid_source, invalid_source_value, report_provenance, validate_provenance};
use crate::ic::{
    IcHostError, IcNodeStatusObservation, IcNodeStatusScope, IcNodeStatusSnapshot,
    IcNodeStatusSourceData, IcSourceRequest,
    node_status::{
        canonicalize_node_status_rows, node_status_group_counts, validate_default_node_scope,
    },
};

///
/// IcNodeStatusSource
///
/// Source contract for one finite official Dashboard node-status snapshot.
///

pub trait IcNodeStatusSource {
    /// Fetch the Dashboard's default public-mainnet node resource in one request.
    fn fetch_node_status_snapshot(
        &self,
        request: &IcSourceRequest,
    ) -> Result<IcNodeStatusSourceData, IcHostError>;
}

pub(in crate::ic) fn node_status_snapshot_from_source(
    request: &IcSourceRequest,
    mut source: IcNodeStatusSourceData,
) -> Result<IcNodeStatusSnapshot, IcHostError> {
    validate_provenance(request, &source.source)?;
    if source.scope != IcNodeStatusScope::DashboardMainnetDefault {
        return invalid_source(format!(
            "node-status scope is {:?}, expected dashboard_mainnet_default",
            source.scope
        ));
    }
    if source.cloud_engine_nodes_included {
        return invalid_source(
            "default public-mainnet node scope cannot claim cloud-engine node inclusion",
        );
    }
    canonicalize_node_status_rows(&mut source.nodes).map_err(invalid_source_value)?;
    validate_default_node_scope(&source.nodes).map_err(invalid_source_value)?;
    let counts = node_status_group_counts(source.nodes.iter());

    Ok(IcNodeStatusSnapshot {
        observation: IcNodeStatusObservation {
            source: report_provenance(source.source),
            scope: source.scope,
            cloud_engine_nodes_included: source.cloud_engine_nodes_included,
            cache: None,
        },
        node_count: source.nodes.len(),
        counts,
        nodes: source.nodes,
    })
}
