use std::path::{Path, PathBuf};

use super::{
    NnsTopologyHostError,
    request::{
        TopologyRefreshParts, TopologyRequestParts, data_center_list_request,
        data_center_refresh_request, node_list_request, node_operator_list_request,
        node_operator_refresh_request, node_provider_list_request, node_provider_refresh_request,
        node_refresh_request, subnet_catalog_list_request, subnet_catalog_refresh_request,
    },
};
use crate::{
    nns::{
        data_center::report::{
            NnsDataCenterListReport, NnsDataCenterRefreshReport, build_nns_data_center_list_report,
            refresh_nns_data_center_report,
        },
        node::report::{
            NnsNodeListReport, NnsNodeRefreshReport, build_nns_node_list_report,
            refresh_nns_node_report,
        },
        node_operator::report::{
            NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport,
            build_nns_node_operator_list_report, refresh_nns_node_operator_report,
        },
        node_provider::report::{
            NnsNodeProviderListReport, NnsNodeProviderRefreshReport,
            build_nns_node_provider_list_report, refresh_nns_node_provider_report,
        },
    },
    subnet_catalog::{
        SubnetCatalogListReport, SubnetCatalogRefreshReport, build_subnet_catalog_list_report,
        format_utc_timestamp_secs, refresh_subnet_catalog,
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
/// NnsTopologyRefreshSourceRequest
///
/// Source request settings for refreshing one complete NNS topology snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyRefreshSourceRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl NnsTopologyRefreshSourceRequest {
    #[must_use]
    pub fn new(
        icp_root: impl Into<PathBuf>,
        network: impl Into<String>,
        endpoint: impl Into<String>,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            icp_root: icp_root.into(),
            network: network.into(),
            endpoint: endpoint.into(),
            now_unix_secs,
            lock_stale_after_seconds,
            dry_run: false,
            fetched_at: format_utc_timestamp_secs(now_unix_secs),
            fetched_by: "ic-query".to_string(),
        }
    }

    #[must_use]
    pub const fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

impl TopologyRequestParts for NnsTopologyRefreshSourceRequest {
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

impl TopologyRefreshParts for NnsTopologyRefreshSourceRequest {
    fn lock_stale_after_seconds(&self) -> u64 {
        self.lock_stale_after_seconds
    }

    fn dry_run(&self) -> bool {
        self.dry_run
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
/// NnsTopologyRefreshSource
///
/// Source contract for refreshing complete NNS topology component reports.
///

pub trait NnsTopologyRefreshSource {
    fn refresh_subnet_catalog_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<SubnetCatalogRefreshReport, NnsTopologyHostError>;

    fn refresh_node_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeRefreshReport, NnsTopologyHostError>;

    fn refresh_node_provider_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeProviderRefreshReport, NnsTopologyHostError>;

    fn refresh_node_operator_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeOperatorRefreshReport, NnsTopologyHostError>;

    fn refresh_data_center_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsDataCenterRefreshReport, NnsTopologyHostError>;
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

impl NnsTopologyRefreshSource for LiveNnsTopologySource {
    fn refresh_subnet_catalog_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<SubnetCatalogRefreshReport, NnsTopologyHostError> {
        Ok(refresh_subnet_catalog(&subnet_catalog_refresh_request(
            request,
        ))?)
    }

    fn refresh_node_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeRefreshReport, NnsTopologyHostError> {
        Ok(refresh_nns_node_report(&node_refresh_request(request))?)
    }

    fn refresh_node_provider_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeProviderRefreshReport, NnsTopologyHostError> {
        Ok(refresh_nns_node_provider_report(
            &node_provider_refresh_request(request),
        )?)
    }

    fn refresh_node_operator_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeOperatorRefreshReport, NnsTopologyHostError> {
        Ok(refresh_nns_node_operator_report(
            &node_operator_refresh_request(request),
        )?)
    }

    fn refresh_data_center_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsDataCenterRefreshReport, NnsTopologyHostError> {
        Ok(refresh_nns_data_center_report(
            &data_center_refresh_request(request),
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

pub(super) fn topology_refresh_source_request_from(
    request: &impl TopologyRefreshParts,
) -> NnsTopologyRefreshSourceRequest {
    NnsTopologyRefreshSourceRequest::new(
        request.icp_root(),
        request.network(),
        request.source_endpoint(),
        request.now_unix_secs(),
        request.lock_stale_after_seconds(),
    )
    .with_dry_run(request.dry_run())
}
