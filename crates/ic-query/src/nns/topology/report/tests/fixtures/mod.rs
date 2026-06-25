mod data_center;
mod node;
mod node_operator;
mod node_provider;
mod subnet;

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
