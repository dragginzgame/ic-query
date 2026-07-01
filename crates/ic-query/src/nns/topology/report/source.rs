use std::path::{Path, PathBuf};

use super::{
    NnsTopologyHostError,
    request::{
        TopologyRequestParts, data_center_list_request, node_list_request,
        node_operator_list_request, node_provider_list_request, subnet_catalog_list_request,
    },
};
use crate::{
    nns::{
        data_center::report::{NnsDataCenterListReport, build_nns_data_center_list_report},
        node::report::{NnsNodeListReport, build_nns_node_list_report},
        node_operator::report::{NnsNodeOperatorListReport, build_nns_node_operator_list_report},
        node_provider::report::{NnsNodeProviderListReport, build_nns_node_provider_list_report},
    },
    subnet_catalog::{
        SubnetCatalogListReport, build_subnet_catalog_list_report, format_utc_timestamp_secs,
    },
};

///
/// NnsTopologySourceRequest
///
/// Source request settings for fetching one complete NNS topology input snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologySourceRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub endpoint: String,
    pub now_unix_secs: u64,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl NnsTopologySourceRequest {
    #[must_use]
    pub fn new(
        icp_root: impl Into<PathBuf>,
        network: impl Into<String>,
        endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            icp_root: icp_root.into(),
            network: network.into(),
            endpoint: endpoint.into(),
            now_unix_secs,
            fetched_at: format_utc_timestamp_secs(now_unix_secs),
            fetched_by: "ic-query".to_string(),
        }
    }
}

impl TopologyRequestParts for NnsTopologySourceRequest {
    fn icp_root(&self) -> &Path {
        &self.icp_root
    }

    fn network(&self) -> &str {
        &self.network
    }

    fn source_endpoint(&self) -> &str {
        &self.endpoint
    }

    fn now_unix_secs(&self) -> u64 {
        self.now_unix_secs
    }
}

///
/// NnsTopologySource
///
/// Source contract for fetching complete NNS topology component reports.
///

pub trait NnsTopologySource {
    fn fetch_subnet_catalog_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<SubnetCatalogListReport, NnsTopologyHostError>;

    fn fetch_node_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeListReport, NnsTopologyHostError>;

    fn fetch_node_provider_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeProviderListReport, NnsTopologyHostError>;

    fn fetch_node_operator_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsTopologyHostError>;

    fn fetch_data_center_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsDataCenterListReport, NnsTopologyHostError>;
}

///
/// LiveNnsTopologySource
///
/// Source implementation backed by the built-in NNS topology component reports.
///

pub struct LiveNnsTopologySource;

impl NnsTopologySource for LiveNnsTopologySource {
    fn fetch_subnet_catalog_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<SubnetCatalogListReport, NnsTopologyHostError> {
        Ok(build_subnet_catalog_list_report(
            &subnet_catalog_list_request(request),
        )?)
    }

    fn fetch_node_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeListReport, NnsTopologyHostError> {
        Ok(build_nns_node_list_report(&node_list_request(request))?)
    }

    fn fetch_node_provider_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeProviderListReport, NnsTopologyHostError> {
        Ok(build_nns_node_provider_list_report(
            &node_provider_list_request(request),
        )?)
    }

    fn fetch_node_operator_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsTopologyHostError> {
        Ok(build_nns_node_operator_list_report(
            &node_operator_list_request(request),
        )?)
    }

    fn fetch_data_center_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsDataCenterListReport, NnsTopologyHostError> {
        Ok(build_nns_data_center_list_report(
            &data_center_list_request(request),
        )?)
    }
}

pub(super) fn topology_source_request_from(
    request: &impl TopologyRequestParts,
) -> NnsTopologySourceRequest {
    NnsTopologySourceRequest::new(
        request.icp_root(),
        request.network(),
        request.source_endpoint(),
        request.now_unix_secs(),
    )
}
