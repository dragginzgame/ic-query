//! Module: nns::topology::report::text
//!
//! Responsibility: expose NNS topology text renderers.
//! Does not own: report construction, cache loading, or JSON output.
//! Boundary: groups human-facing topology report renderers by report type.

mod capacity;
mod common;
mod coverage;
mod gaps;
mod health;
mod providers;
mod refresh;
mod regions;
mod summary;
mod versions;

pub use capacity::nns_topology_capacity_report_text;
pub use coverage::nns_topology_coverage_report_text;
pub use gaps::nns_topology_gaps_report_text;
pub use health::nns_topology_health_report_text;
pub use providers::nns_topology_providers_report_text;
pub use refresh::nns_topology_refresh_report_text;
pub use regions::nns_topology_regions_report_text;
pub use summary::nns_topology_summary_report_text;
pub use versions::nns_topology_versions_report_text;
