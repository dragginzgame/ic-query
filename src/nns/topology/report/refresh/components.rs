use crate::{
    nns::{
        data_center::report::NnsDataCenterRefreshReport, node::report::NnsNodeRefreshReport,
        node_operator::report::NnsNodeOperatorRefreshReport,
        node_provider::report::NnsNodeProviderRefreshReport,
    },
    subnet_catalog::SubnetCatalogRefreshReport,
};

pub(in crate::nns::topology::report) struct NnsTopologyRefreshComponentReports {
    pub(in crate::nns::topology::report) subnet: SubnetCatalogRefreshReport,
    pub(in crate::nns::topology::report) node: NnsNodeRefreshReport,
    pub(in crate::nns::topology::report) node_provider: NnsNodeProviderRefreshReport,
    pub(in crate::nns::topology::report) node_operator: NnsNodeOperatorRefreshReport,
    pub(in crate::nns::topology::report) data_center: NnsDataCenterRefreshReport,
}
