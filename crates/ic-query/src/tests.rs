use std::path::PathBuf;

pub fn temp_dir(prefix: &str) -> PathBuf {
    let parent = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .join("target/tmp");
    std::fs::create_dir_all(&parent).expect("create workspace test scratch");
    ic_host_fs::testing::create_fixture_directory(&parent, prefix)
        .expect("create private fixture directory")
}

#[cfg(feature = "nns-host")]
impl From<crate::nns::node::NnsNodeListReport> for crate::nns::NnsRegistryInventorySnapshot {
    fn from(report: crate::nns::node::NnsNodeListReport) -> Self {
        Self {
            schema_version: report.schema_version,
            provenance: report.provenance,
            nodes: report.nodes,
            node_operators: Vec::new(),
            data_centers: Vec::new(),
        }
    }
}

#[cfg(feature = "nns-host")]
impl From<crate::nns::node_operator::NnsNodeOperatorListReport>
    for crate::nns::NnsRegistryInventorySnapshot
{
    fn from(report: crate::nns::node_operator::NnsNodeOperatorListReport) -> Self {
        Self {
            schema_version: report.schema_version,
            provenance: report.provenance,
            node_operators: report.node_operators,
            nodes: Vec::new(),
            data_centers: Vec::new(),
        }
    }
}

#[cfg(feature = "nns-host")]
impl From<crate::nns::data_center::NnsDataCenterListReport>
    for crate::nns::NnsRegistryInventorySnapshot
{
    fn from(report: crate::nns::data_center::NnsDataCenterListReport) -> Self {
        Self {
            schema_version: report.schema_version,
            provenance: report.provenance,
            data_centers: report.data_centers,
            nodes: Vec::new(),
            node_operators: Vec::new(),
        }
    }
}

#[cfg(feature = "nns-host")]
pub fn inventory_for_provider_rows(
    request: &crate::report::CollectionSourceRequest,
    providers: &[crate::nns::node_provider::NnsNodeProviderRow],
) -> crate::nns::NnsRegistryInventorySnapshot {
    let nodes = providers
        .iter()
        .flat_map(|provider| {
            (0..provider.node_count.unwrap_or(0)).map(move |index| crate::nns::node::NnsNodeRow {
                node_principal: format!("fixture-{index}-{}", provider.node_provider_principal),
                node_operator_principal: "fixture-operator".to_string(),
                node_provider_principal: provider.node_provider_principal.clone(),
                subnet_principal: "fixture-subnet".to_string(),
                subnet_kind: crate::subnet_catalog::SubnetKind::Application,
                data_center_id: "fixture".to_string(),
            })
        })
        .collect();
    crate::nns::NnsRegistryInventorySnapshot {
        schema_version: 1,
        provenance: crate::nns::registry::NnsRegistryInventoryProvenance {
            network: request.network.clone(),
            registry_canister_id: crate::subnet_catalog::MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            source_endpoint: request.endpoint.clone(),
            fetched_by: request.fetched_by.clone(),
        },
        nodes,
        node_operators: Vec::new(),
        data_centers: Vec::new(),
    }
}
