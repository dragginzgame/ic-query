//! NNS node-provider inventory requests, reports, host builders.

mod report;

pub use report::*;

#[cfg(feature = "nns-host")]
pub(in crate::nns) use report::{load_or_refresh_metadata, node_provider_report_from_snapshot};
