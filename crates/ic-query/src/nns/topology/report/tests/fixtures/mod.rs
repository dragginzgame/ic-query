mod data_center;
mod node;
mod node_operator;
mod node_provider;
mod subnet;

use crate::{
    nns::{
        data_center::NnsDataCenterListReport,
        node::{NnsNodeHostError, NnsNodeListReport},
        node_operator::NnsNodeOperatorListReport,
        node_provider::NnsNodeProviderListReport,
        topology::report::{
            NnsTopologyHostError, NnsTopologyReadRequest, NnsTopologySource,
            NnsTopologySourceRequest,
        },
    },
    subnet_catalog::SubnetCatalogListReport,
};
use std::cell::RefCell;

pub(in crate::nns::topology::report::tests) use data_center::{
    complete_data_center_report_fixture, data_center_refresh_report_fixture,
    data_center_report_fixture, dry_run_data_center_refresh_report_fixture,
};
pub(in crate::nns::topology::report::tests) use node::{
    dry_run_node_refresh_report_fixture, node_refresh_report_fixture, node_report_fixture,
};
pub(in crate::nns::topology::report::tests) use node_operator::{
    complete_node_operator_report_fixture, dry_run_node_operator_refresh_report_fixture,
    node_operator_refresh_report_fixture, node_operator_report_fixture,
};
pub(in crate::nns::topology::report::tests) use node_provider::{
    complete_node_provider_report_fixture, dry_run_node_provider_refresh_report_fixture,
    node_provider_refresh_report_fixture, node_provider_report_fixture,
};
pub(in crate::nns::topology::report::tests) use subnet::{
    dry_run_subnet_refresh_report_fixture, subnet_refresh_report_fixture, subnet_report_fixture,
};

pub(in crate::nns::topology::report::tests) fn topology_read_request_fixture()
-> NnsTopologyReadRequest {
    NnsTopologyReadRequest::new("/cache", "ic", "https://icp-api.io", 1_780_531_200)
}

///
/// RecordingTopologySource
///
/// Fixture component reports with observable read order and injected source failure.
///

#[derive(Default)]
pub(in crate::nns::topology::report::tests) struct RecordingTopologySource {
    pub(in crate::nns::topology::report::tests) calls: RefCell<Vec<&'static str>>,
    pub(in crate::nns::topology::report::tests) fail_at: Option<&'static str>,
}

impl RecordingTopologySource {
    fn record(
        &self,
        component: &'static str,
        request: &NnsTopologySourceRequest,
    ) -> Result<(), NnsTopologyHostError> {
        assert_eq!(
            request,
            &NnsTopologySourceRequest::new("/cache", "ic", "https://icp-api.io", 1_780_531_200)
        );
        self.calls.borrow_mut().push(component);
        if self.fail_at == Some(component) {
            return Err(NnsNodeHostError::InvalidSourceData {
                reason: component.to_string(),
            }
            .into());
        }
        Ok(())
    }
}

impl NnsTopologySource for RecordingTopologySource {
    fn fetch_subnet_catalog_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<SubnetCatalogListReport, NnsTopologyHostError> {
        self.record("subnet_catalog", request)?;
        Ok(subnet_report_fixture())
    }

    fn fetch_node_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeListReport, NnsTopologyHostError> {
        self.record("nodes", request)?;
        Ok(node_report_fixture())
    }

    fn fetch_node_provider_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeProviderListReport, NnsTopologyHostError> {
        self.record("node_providers", request)?;
        Ok(node_provider_report_fixture())
    }

    fn fetch_node_operator_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsTopologyHostError> {
        self.record("node_operators", request)?;
        Ok(node_operator_report_fixture())
    }

    fn fetch_data_center_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsDataCenterListReport, NnsTopologyHostError> {
        self.record("data_centers", request)?;
        Ok(data_center_report_fixture())
    }
}
