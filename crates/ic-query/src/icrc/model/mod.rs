//! Module: icrc::model
//!
//! Responsibility: expose typed ICRC requests, reports, source data, and errors.
//! Does not own: Clap parsing, live transport, report construction, or rendering.
//! Boundary: keeps public contracts, host-source data, errors, and validation separate.

mod contracts;
#[cfg(feature = "host")]
mod data;
mod error;
#[cfg(feature = "host")]
mod subaccount;

pub use contracts::*;
#[cfg(feature = "host")]
pub use data::*;
pub use error::IcrcError;
#[cfg(feature = "host")]
pub(in crate::icrc) use subaccount::{normalize_subaccount_hex, subaccount_bytes_from_hex};
