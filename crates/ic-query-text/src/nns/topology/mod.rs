//! Module: nns::topology
//!
//! Responsibility: expose NNS topology text renderers.
//! Does not own: report construction, cache loading, or JSON output.
//! Boundary: groups human-facing topology report renderers by report type.

mod capacity;
mod check;
mod common;
mod coverage;
mod gaps;
mod percent;
mod providers;
mod refresh;
mod regions;
mod summary;
mod versions;

pub use crate::nns::topology::capacity::nns_topology_capacity_report_text;
pub use crate::nns::topology::check::nns_topology_check_report_text;
pub use crate::nns::topology::coverage::nns_topology_coverage_report_text;
pub use crate::nns::topology::gaps::nns_topology_gaps_report_text;
pub use crate::nns::topology::providers::nns_topology_providers_report_text;
pub use crate::nns::topology::refresh::nns_topology_refresh_report_text;
pub use crate::nns::topology::regions::nns_topology_regions_report_text;
pub use crate::nns::topology::summary::nns_topology_summary_report_text;
pub use crate::nns::topology::versions::nns_topology_versions_report_text;

mod subnet_topology;
pub use subnet_topology::nns_subnet_topology_report_text;

const COMPACT_PRINCIPAL_CHARS: usize = 12;
