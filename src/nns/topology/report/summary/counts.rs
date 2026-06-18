//! Module: nns::topology::report::summary::counts
//!
//! Responsibility: count NNS topology summary entities by subnet kind.
//! Does not own: relation joins, registry-version rows, or text rendering.
//! Boundary: provides focused count helpers for summary construction.

use crate::{
    nns::node::report::NnsNodeListReport,
    subnet_catalog::{SubnetCatalogListReport, SubnetKind},
};

pub(super) fn subnet_count_by_kind(report: &SubnetCatalogListReport, kind: SubnetKind) -> usize {
    report
        .subnets
        .iter()
        .filter(|subnet| subnet.subnet_kind == kind)
        .count()
}

pub(super) fn node_count_by_subnet_kind(report: &NnsNodeListReport, kind: &str) -> usize {
    report
        .nodes
        .iter()
        .filter(|node| node.subnet_kind.eq_ignore_ascii_case(kind))
        .count()
}
