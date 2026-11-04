//! Module: icrc::model::contracts::requests::account
//!
//! Responsibility: ICRC balance and allowance request contracts.
//! Does not own: account-history pagination, ledger history, live transport, or reports.
//! Boundary: keeps structured account and spender selection explicit.

use super::IcrcLedgerRequest;

///
/// IcrcBalanceRequest
///
/// Request accepted by the generic ICRC account balance report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcBalanceRequest {
    /// Shared ledger identity, explicit endpoint and caller collection clock.
    pub ledger: IcrcLedgerRequest,
    pub account_owner: String,
    pub subaccount_hex: Option<String>,
}

impl IcrcBalanceRequest {
    #[must_use]
    pub fn new(ledger: IcrcLedgerRequest, account_owner: impl Into<String>) -> Self {
        Self {
            ledger,
            account_owner: account_owner.into(),
            subaccount_hex: None,
        }
    }
}

///
/// IcrcAllowanceRequest
///
/// Request accepted by the generic ICRC allowance report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcAllowanceRequest {
    /// Shared ledger identity, explicit endpoint and caller collection clock.
    pub ledger: IcrcLedgerRequest,
    pub account_owner: String,
    pub account_subaccount_hex: Option<String>,
    pub spender_owner: String,
    pub spender_subaccount_hex: Option<String>,
}

impl IcrcAllowanceRequest {
    #[must_use]
    pub fn new(
        ledger: IcrcLedgerRequest,
        account_owner: impl Into<String>,
        spender_owner: impl Into<String>,
    ) -> Self {
        Self {
            ledger,
            account_owner: account_owner.into(),
            account_subaccount_hex: None,
            spender_owner: spender_owner.into(),
            spender_subaccount_hex: None,
        }
    }
}
