//! Human-facing presentation of typed IC Query reports.
//!
//! Renderers accept collected reports and return text. Collection, validation,
//! JSON contracts and caches remain in `ic-query`; process IO belongs to the CLI.

pub mod cloud_engine;
mod human_quantity;
pub mod ic;
pub mod icrc;
pub mod nns;
mod report;
pub mod sns;
/// Subnet catalog lists, details and refresh summaries.
#[cfg(feature = "subnet-catalog-host")]
pub mod subnet_catalog;
pub mod system;
mod text_value;
mod token_amount;
mod token_metadata_text;

#[cfg(test)]
mod tests;

pub use report::TextOptions;
