//! Module: icrc::model
//!
//! Responsibility: expose typed ICRC requests, reports, source data, and errors.
//! Does not own: Clap parsing, live transport, report construction, or rendering.
//! Boundary: keeps public contracts, host-source data, errors, and validation separate.

#[cfg(feature = "icrc-host")]
mod data;
mod error;
mod reports;
mod requests;
mod subaccount;
mod token_metadata;

#[cfg(feature = "icrc-host")]
pub use data::{
    CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionCollectionData,
    IcrcCapabilitiesData,
};
pub use error::{IcrcAccountTransactionError, IcrcError};
pub use reports::*;
pub use requests::*;
pub use subaccount::normalize_subaccount_hex;
#[cfg(feature = "icrc-host")]
pub(in crate::icrc) use subaccount::{
    normalize_optional_subaccount_hex, subaccount_bytes_from_hex,
};

pub use token_metadata::{
    IcrcIndexData, IcrcMetadataValueKind, IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenStandardRow,
};
