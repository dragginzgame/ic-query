//! Module: sns::report::live::types::token
//!
//! Responsibility: SNS ledger and index Candid wire types.
//! Does not own: live transport, token conversion, amount formatting, or rendering.
//! Boundary: mirrors token metadata, account, standard, and index payloads.

use candid::{CandidType, Deserialize, Int, Nat, Principal};

///
/// IcrcAccount
///
/// Candid ICRC account used by SNS ledger metadata.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct IcrcAccount {
    pub(in crate::sns::report::live) owner: Principal,
    pub(in crate::sns::report::live) subaccount: Option<Vec<u8>>,
}

///
/// IcrcMetadataValue
///
/// Candid ICRC metadata value returned by SNS ledgers.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report) enum IcrcMetadataValue {
    Nat(Nat),
    Int(Int),
    Text(String),
    Blob(Vec<u8>),
}

///
/// IcrcSupportedStandard
///
/// Candid ICRC standard descriptor returned by SNS ledgers.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct IcrcSupportedStandard {
    pub(in crate::sns::report::live) name: String,
    pub(in crate::sns::report::live) url: String,
}

///
/// GetIndexPrincipalResult
///
/// Candid result returned by SNS ledger index discovery.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum GetIndexPrincipalResult {
    Ok(Principal),
    Err(GetIndexPrincipalError),
}

///
/// GetIndexPrincipalError
///
/// Candid error returned by SNS ledger index discovery.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum GetIndexPrincipalError {
    IndexPrincipalNotSet,
    GenericError {
        error_code: Nat,
        description: String,
    },
}
