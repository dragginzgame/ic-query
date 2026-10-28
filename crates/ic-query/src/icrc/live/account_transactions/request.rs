//! Shared admission of account-history page bounds and target identities.

use crate::icrc::{
    IcrcAccountTransactionError, IcrcError, ledger::principal_from_text,
    live::ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE,
};

pub(in crate::icrc) fn validate_account_transaction_page_size(
    page_size: u32,
) -> Result<(), IcrcAccountTransactionError> {
    if !(1..=ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE).contains(&page_size) {
        return Err(IcrcAccountTransactionError::InvalidPageSize {
            page_size,
            max_page_size: ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE,
        });
    }
    Ok(())
}

pub(in crate::icrc) fn normalize_account_transaction_identity(
    ledger_canister_id: &str,
    account_owner: &str,
) -> Result<(String, String), IcrcError> {
    let ledger = principal_from_text(ledger_canister_id, "ledger_canister_id")?;
    let owner = principal_from_text(account_owner, "account_owner")?;
    Ok((ledger.to_text(), owner.to_text()))
}

pub(in crate::icrc) fn normalize_optional_index_canister_id(
    value: Option<&str>,
) -> Result<Option<String>, IcrcError> {
    value
        .map(|value| principal_from_text(value, "index_canister_id"))
        .transpose()
        .map(|principal| principal.map(|principal| principal.to_text()))
}
