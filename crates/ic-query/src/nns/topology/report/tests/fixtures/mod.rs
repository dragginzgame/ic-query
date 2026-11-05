use crate::nns::topology::{
    NnsTopologyHostError, NnsTopologyRefreshRequest, NnsTopologyRefreshSource, NnsTopologySource,
};
use crate::nns::{
    NnsInventoryCollection, NnsInventoryError, NnsInventoryListRequest,
    NnsRegistryInventorySnapshot, NnsRegistryInventorySource,
    node_provider::{NnsNodeProviderMetadata, NnsNodeProviderSource},
};
use crate::report::{CollectionSourceRequest, SourceFuture};
use crate::subnet_catalog::{SubnetCatalogListReport, SubnetCatalogRefreshReport};
use std::sync::Mutex;
mod data_center;
mod node;
mod node_operator;
mod node_provider;
mod subnet;
pub(in crate::nns::topology::report::tests) use data_center::{
    complete_data_center_report_fixture, data_center_report_fixture,
};
pub(in crate::nns::topology::report::tests) use node::node_report_fixture;
pub(in crate::nns::topology::report::tests) use node_operator::{
    complete_node_operator_report_fixture, node_operator_report_fixture,
};
pub(in crate::nns::topology::report::tests) use node_provider::{
    complete_node_provider_report_fixture, node_provider_report_fixture,
};
pub(in crate::nns::topology::report::tests) use subnet::{
    dry_run_subnet_refresh_report_fixture, subnet_refresh_report_fixture, subnet_report_fixture,
};

pub(in crate::nns::topology::report::tests) fn topology_read_request_fixture()
-> NnsInventoryListRequest {
    NnsInventoryListRequest::new(
        crate::nns::NnsInventoryCacheRequest::new(
            crate::tests::temp_dir("ic-query-topology"),
            "ic",
        ),
        "https://icp-api.io",
        1_780_531_200,
    )
}

///
/// RecordingTopologySource
///
/// Observable raw acquisitions with independently injected Registry and Governance failures.
///

#[derive(Default)]
pub(in crate::nns::topology::report::tests) struct RecordingTopologySource {
    pub(in crate::nns::topology::report::tests) calls: Mutex<Vec<&'static str>>,
    pub(in crate::nns::topology::report::tests) fail_at: Option<&'static str>,
    pub(in crate::nns::topology::report::tests) invalid_inventory: bool,
}
impl RecordingTopologySource {
    fn record(&self, component: &'static str) -> Result<(), NnsInventoryError> {
        self.calls.lock().unwrap().push(component);
        if self.fail_at == Some(component) {
            return Err(NnsInventoryError::InvalidSourceData {
                collection: NnsInventoryCollection::RegistryInventory,
                reason: component.to_string(),
            });
        }
        Ok(())
    }
}
impl NnsRegistryInventorySource for RecordingTopologySource {
    fn fetch_registry_inventory<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> SourceFuture<'a, Result<NnsRegistryInventorySnapshot, NnsInventoryError>> {
        Box::pin(async move {
            self.record("inventory")?;
            let node = node_report_fixture();
            let operator = node_operator_report_fixture();
            let dc = data_center_report_fixture();
            let mut provenance = node.provenance;
            provenance.network.clone_from(&request.network);
            provenance.source_endpoint.clone_from(&request.endpoint);
            provenance.fetched_at.clone_from(&request.fetched_at);
            Ok(NnsRegistryInventorySnapshot {
                schema_version: u32::from(!self.invalid_inventory),
                provenance,
                nodes: node.nodes,
                node_operators: operator.node_operators,
                data_centers: dc.data_centers,
            })
        })
    }
}
impl NnsNodeProviderSource for RecordingTopologySource {
    fn fetch_node_provider_metadata<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> SourceFuture<'a, Result<NnsNodeProviderMetadata, NnsInventoryError>> {
        Box::pin(async move {
            self.record("metadata")?;
            let report = node_provider_report_fixture();
            let mut node_providers = report.node_providers;
            for row in &mut node_providers {
                row.node_count = None;
            }
            Ok(NnsNodeProviderMetadata {
                schema_version: 1,
                source: request.clone(),
                governance_canister_id: report.governance_canister_id,
                node_providers,
            })
        })
    }
}
impl NnsTopologySource for RecordingTopologySource {
    fn fetch_subnet_catalog_list_report<'a>(
        &'a self,
        _request: &'a NnsInventoryListRequest,
    ) -> SourceFuture<'a, Result<SubnetCatalogListReport, NnsTopologyHostError>> {
        Box::pin(async move {
            self.record("subnet_catalog")?;
            Ok(subnet_report_fixture())
        })
    }
}
impl NnsTopologyRefreshSource for RecordingTopologySource {
    fn refresh_subnet_catalog_report<'a>(
        &'a self,
        request: &'a NnsTopologyRefreshRequest,
    ) -> SourceFuture<'a, Result<SubnetCatalogRefreshReport, NnsTopologyHostError>> {
        Box::pin(async move {
            self.record("subnet_catalog")?;
            Ok(if request.dry_run {
                dry_run_subnet_refresh_report_fixture()
            } else {
                subnet_refresh_report_fixture()
            })
        })
    }
}

fn inventory_provenance(
    registry_version: u64,
    fetched_at: &str,
) -> crate::nns::registry::NnsRegistryInventoryProvenance {
    crate::nns::registry::NnsRegistryInventoryProvenance {
        network: crate::subnet_catalog::MAINNET_NETWORK.to_string(),
        registry_canister_id: crate::subnet_catalog::MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: fetched_at.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
    }
}
