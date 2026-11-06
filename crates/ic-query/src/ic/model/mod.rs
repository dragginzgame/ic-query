//! Module: ic::model
//!
//! Responsibility: expose typed Dashboard requests, reports, source data, and errors.
//! Does not own: HTTP transport, source validation, report assembly, or rendering.
//! Boundary: keeps caller intent, serialized evidence, and host contracts separate.

#[cfg(feature = "dashboard-host")]
mod data;
#[cfg(feature = "dashboard-host")]
mod error;
mod reports;
mod requests;

#[cfg(feature = "dashboard-host")]
pub use data::*;
#[cfg(feature = "dashboard-host")]
pub use error::IcHostError;
pub use reports::*;
pub use requests::*;
