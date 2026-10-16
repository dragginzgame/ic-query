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

    #[must_use]
    pub fn with_subaccount_hex(mut self, subaccount_hex: impl Into<String>) -> Self {
        self.subaccount_hex = Some(subaccount_hex.into());
        self
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

    #[must_use]
    pub fn with_account_subaccount_hex(
        mut self,
        account_subaccount_hex: impl Into<String>,
    ) -> Self {
        self.account_subaccount_hex = Some(account_subaccount_hex.into());
        self
    }

    #[must_use]
    pub fn with_spender_subaccount_hex(
        mut self,
        spender_subaccount_hex: impl Into<String>,
    ) -> Self {
        self.spender_subaccount_hex = Some(spender_subaccount_hex.into());
        self
    }
}
