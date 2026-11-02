//! Text presentation of NNS reports.
/// Registry data center reports.
pub mod data_center;
pub mod governance;
pub mod neuron;
/// Registry node reports.
pub mod node;
/// Registry node operator reports.
pub mod node_operator;
/// Node provider inventories and details.
pub mod node_provider;
pub mod proposals;
/// Certified Registry version reports.
pub mod registry;
mod render;
pub mod topology;
#[cfg(feature = "nns-host")]
pub use render::nns_inventory_refresh_report_text;
