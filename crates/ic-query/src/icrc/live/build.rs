//! Module: icrc::live::build
//!
//! Responsibility: build public ICRC reports from source data.
//! Does not own: Candid calls, source implementations, command parsing, or rendering.
//! Boundary: validates request-local values and projects raw source data into reports.

use super::{
    ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE, ICRC_ACCOUNT_TRANSACTION_PAGE_REPORT_SCHEMA_VERSION,
    ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION, ICRC_ARCHIVES_REPORT_SCHEMA_VERSION,
    ICRC_BALANCE_REPORT_SCHEMA_VERSION, ICRC_BLOCK_TYPES_REPORT_SCHEMA_VERSION,
    ICRC_CAPABILITIES_REPORT_SCHEMA_VERSION, ICRC_FETCHED_BY, ICRC_INDEX_REPORT_SCHEMA_VERSION,
    ICRC_TIP_CERTIFICATE_REPORT_SCHEMA_VERSION, ICRC_TOKEN_REPORT_SCHEMA_VERSION,
    ICRC_TRANSACTIONS_REPORT_SCHEMA_VERSION, IcrcAccountTransactionPageSource, IcrcAllowanceSource,
    IcrcArchivesSource, IcrcBalanceSource, IcrcBlockTypesSource, IcrcCapabilitiesSource,
    IcrcIndexSource, IcrcTipCertificateSource, IcrcTokenSource, IcrcTransactionsSource,
    LiveIcrcSource,
    account_transactions::{normalize_transaction_cursor, validate_canonical_account_transactions},
};
use crate::{
    icrc::{
        ledger::principal_from_text,
        model::{
            IcrcAccountTransactionError, IcrcAccountTransactionPageReport,
            IcrcAccountTransactionPageRequest, IcrcAllowanceReport, IcrcAllowanceRequest,
            IcrcArchivesReport, IcrcArchivesRequest, IcrcBalanceReport, IcrcBalanceRequest,
            IcrcBlockTypesReport, IcrcCapabilitiesReport, IcrcError, IcrcIndexReport,
            IcrcLedgerRequest, IcrcTipCertificateReport, IcrcTokenReport, IcrcTransactionsReport,
            IcrcTransactionsRequest, normalize_optional_subaccount_hex,
        },
    },
    subnet_catalog::format_utc_timestamp_secs,
};

pub fn build_icrc_token_report(request: &IcrcLedgerRequest) -> Result<IcrcTokenReport, IcrcError> {
    build_icrc_token_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_balance_report(
    request: &IcrcBalanceRequest,
) -> Result<IcrcBalanceReport, IcrcError> {
    build_icrc_balance_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_allowance_report(
    request: &IcrcAllowanceRequest,
) -> Result<IcrcAllowanceReport, IcrcError> {
    build_icrc_allowance_report_with_source(request, &LiveIcrcSource)
}

/// Resolves an ICRC index and builds one account-transaction page.
pub fn build_icrc_account_transaction_page_report(
    request: &IcrcAccountTransactionPageRequest,
) -> Result<IcrcAccountTransactionPageReport, IcrcAccountTransactionError> {
    build_icrc_account_transaction_page_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_index_report(request: &IcrcLedgerRequest) -> Result<IcrcIndexReport, IcrcError> {
    build_icrc_index_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_transactions_report(
    request: &IcrcTransactionsRequest,
) -> Result<IcrcTransactionsReport, IcrcError> {
    build_icrc_transactions_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_block_types_report(
    request: &IcrcLedgerRequest,
) -> Result<IcrcBlockTypesReport, IcrcError> {
    build_icrc_block_types_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_archives_report(
    request: &IcrcArchivesRequest,
) -> Result<IcrcArchivesReport, IcrcError> {
    build_icrc_archives_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_tip_certificate_report(
    request: &IcrcLedgerRequest,
) -> Result<IcrcTipCertificateReport, IcrcError> {
    build_icrc_tip_certificate_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_capabilities_report(
    request: &IcrcLedgerRequest,
) -> Result<IcrcCapabilitiesReport, IcrcError> {
    build_icrc_capabilities_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_token_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcTokenSource,
) -> Result<IcrcTokenReport, IcrcError> {
    let token = source.fetch_token(request)?;
    Ok(IcrcTokenReport {
        schema_version: ICRC_TOKEN_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id.clone(),
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: ICRC_FETCHED_BY.to_string(),
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        supported_standards: token.supported_standards,
        metadata: token.metadata,
    })
}

pub fn build_icrc_balance_report_with_source(
    request: &IcrcBalanceRequest,
    source: &dyn IcrcBalanceSource,
) -> Result<IcrcBalanceReport, IcrcError> {
    let request = IcrcBalanceRequest {
        subaccount_hex: normalize_optional_subaccount_hex(request.subaccount_hex.as_deref())?,
        ..request.clone()
    };
    let balance = source.fetch_balance(&request)?;
    Ok(IcrcBalanceReport {
        schema_version: ICRC_BALANCE_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id,
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint,
        fetched_by: ICRC_FETCHED_BY.to_string(),
        token_symbol: balance.token_symbol,
        decimals: balance.decimals,
        balance: balance.balance,
    })
}

pub fn build_icrc_allowance_report_with_source(
    request: &IcrcAllowanceRequest,
    source: &dyn IcrcAllowanceSource,
) -> Result<IcrcAllowanceReport, IcrcError> {
    let request = IcrcAllowanceRequest {
        account_subaccount_hex: normalize_optional_subaccount_hex(
            request.account_subaccount_hex.as_deref(),
        )?,
        spender_subaccount_hex: normalize_optional_subaccount_hex(
            request.spender_subaccount_hex.as_deref(),
        )?,
        ..request.clone()
    };
    let allowance = source.fetch_allowance(&request)?;
    Ok(IcrcAllowanceReport {
        schema_version: ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id,
        account_owner: request.account_owner,
        account_subaccount_hex: request.account_subaccount_hex,
        spender_owner: request.spender_owner,
        spender_subaccount_hex: request.spender_subaccount_hex,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint,
        fetched_by: ICRC_FETCHED_BY.to_string(),
        token_symbol: allowance.token_symbol,
        decimals: allowance.decimals,
        allowance: allowance.allowance,
        expires_at_unix_nanos: allowance.expires_at_unix_nanos,
    })
}

/// Builds one account-transaction page from a caller-supplied index capability.
pub fn build_icrc_account_transaction_page_report_with_source(
    request: &IcrcAccountTransactionPageRequest,
    source: &dyn IcrcAccountTransactionPageSource,
) -> Result<IcrcAccountTransactionPageReport, IcrcAccountTransactionError> {
    if !(1..=ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE).contains(&request.limit) {
        return Err(IcrcAccountTransactionError::InvalidPageSize {
            page_size: request.limit,
            max_page_size: ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE,
        });
    }
    let ledger_canister_id =
        principal_from_text::<IcrcError>(&request.ledger_canister_id, "ledger_canister_id")?
            .to_text();
    let account_owner =
        principal_from_text::<IcrcError>(&request.account_owner, "account_owner")?.to_text();
    let index_canister_id = request
        .index_canister_id
        .as_deref()
        .map(|value| principal_from_text::<IcrcError>(value, "index_canister_id"))
        .transpose()?
        .map(|principal| principal.to_text());
    let request = IcrcAccountTransactionPageRequest {
        ledger_canister_id,
        index_canister_id,
        account_owner,
        subaccount_hex: normalize_optional_subaccount_hex(request.subaccount_hex.as_deref())?,
        start: request
            .start
            .as_deref()
            .map(normalize_transaction_cursor)
            .transpose()?,
        ..request.clone()
    };
    let transactions = source.fetch_account_transaction_page(&request)?;
    let actual_index =
        principal_from_text::<IcrcError>(&transactions.index_canister_id, "index_canister_id")?
            .to_text();
    if let Some(expected_index) = request.index_canister_id.as_deref()
        && expected_index != actual_index
    {
        return Err(IcrcAccountTransactionError::CollectionIndexMismatch {
            expected_index_canister_id: expected_index.to_string(),
            actual_index_canister_id: actual_index,
        });
    }
    if transactions.transactions.len() > usize::try_from(request.limit).unwrap_or(usize::MAX) {
        return Err(IcrcAccountTransactionError::InvalidPage {
            reason: format!(
                "source returned {} transactions for requested limit {}",
                transactions.transactions.len(),
                request.limit
            ),
        });
    }
    let next_start = validate_source_cursor(transactions.next_start.as_deref(), "next_start")?;
    let oldest_transaction_id = validate_source_cursor(
        transactions.oldest_transaction_id.as_deref(),
        "oldest_transaction_id",
    )?;
    validate_canonical_account_transactions(&transactions.transactions)
        .map_err(|reason| IcrcAccountTransactionError::InvalidPage { reason })?;
    let expected_next_start = transactions
        .transactions
        .last()
        .map(|transaction| transaction.id.as_str());
    if next_start.as_deref() != expected_next_start {
        return Err(IcrcAccountTransactionError::InvalidPage {
            reason: "next cursor does not match the oldest returned transaction".to_string(),
        });
    }
    Ok(IcrcAccountTransactionPageReport {
        schema_version: ICRC_ACCOUNT_TRANSACTION_PAGE_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id,
        index_canister_id: actual_index,
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        requested_start: request.start,
        requested_limit: request.limit,
        next_start,
        oldest_transaction_id,
        balance: transactions.balance,
        token_symbol: transactions.token_symbol,
        decimals: transactions.decimals,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint,
        fetched_by: ICRC_FETCHED_BY.to_string(),
        transactions: transactions.transactions,
    })
}

fn validate_source_cursor(
    value: Option<&str>,
    field: &'static str,
) -> Result<Option<String>, IcrcAccountTransactionError> {
    value
        .map(|value| {
            let normalized = normalize_transaction_cursor(value)?;
            if normalized != value {
                return Err(IcrcAccountTransactionError::InvalidPage {
                    reason: format!("{field} {value:?} is not canonical unsigned decimal text"),
                });
            }
            Ok(normalized)
        })
        .transpose()
}

pub fn build_icrc_index_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcIndexSource,
) -> Result<IcrcIndexReport, IcrcError> {
    let index = source.fetch_index(request)?;
    Ok(IcrcIndexReport {
        schema_version: ICRC_INDEX_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id.clone(),
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: ICRC_FETCHED_BY.to_string(),
        index_canister_id: index.index_canister_id,
        index_error: index.index_error,
    })
}

pub fn build_icrc_transactions_report_with_source(
    request: &IcrcTransactionsRequest,
    source: &dyn IcrcTransactionsSource,
) -> Result<IcrcTransactionsReport, IcrcError> {
    let transactions = source.fetch_transactions(request)?;
    Ok(IcrcTransactionsReport {
        schema_version: ICRC_TRANSACTIONS_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id.clone(),
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: ICRC_FETCHED_BY.to_string(),
        requested_start: request.start.to_string(),
        requested_limit: request.limit,
        follow_archives: request.follow_archives,
        log_length: transactions.log_length,
        blocks: transactions.blocks,
        archived_blocks: transactions.archived_blocks,
        followed_archive_blocks: transactions.followed_archive_blocks,
        archive_follow_errors: transactions.archive_follow_errors,
    })
}

pub fn build_icrc_block_types_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcBlockTypesSource,
) -> Result<IcrcBlockTypesReport, IcrcError> {
    let block_types = source.fetch_block_types(request)?;
    Ok(IcrcBlockTypesReport {
        schema_version: ICRC_BLOCK_TYPES_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id.clone(),
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: ICRC_FETCHED_BY.to_string(),
        block_types: block_types.block_types,
    })
}

pub fn build_icrc_archives_report_with_source(
    request: &IcrcArchivesRequest,
    source: &dyn IcrcArchivesSource,
) -> Result<IcrcArchivesReport, IcrcError> {
    let request = IcrcArchivesRequest {
        from_canister_id: request
            .from_canister_id
            .as_deref()
            .map(|canister_id| {
                principal_from_text::<IcrcError>(canister_id, "from_canister_id")
                    .map(|principal| principal.to_text())
            })
            .transpose()?,
        ..request.clone()
    };
    let archives = source.fetch_archives(&request)?;
    Ok(IcrcArchivesReport {
        schema_version: ICRC_ARCHIVES_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id,
        from_canister_id: request.from_canister_id,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint,
        fetched_by: ICRC_FETCHED_BY.to_string(),
        archives: archives.archives,
    })
}

pub fn build_icrc_tip_certificate_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcTipCertificateSource,
) -> Result<IcrcTipCertificateReport, IcrcError> {
    let certificate = source.fetch_tip_certificate(request)?;
    Ok(IcrcTipCertificateReport {
        schema_version: ICRC_TIP_CERTIFICATE_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id.clone(),
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: ICRC_FETCHED_BY.to_string(),
        certificate_present: certificate.certificate_hex.is_some(),
        certificate_hex: certificate.certificate_hex,
        certificate_bytes: certificate.certificate_bytes,
        hash_tree_hex: certificate.hash_tree_hex,
        hash_tree_bytes: certificate.hash_tree_bytes,
    })
}

pub fn build_icrc_capabilities_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcCapabilitiesSource,
) -> Result<IcrcCapabilitiesReport, IcrcError> {
    let capabilities = source.fetch_capabilities(request)?;
    Ok(IcrcCapabilitiesReport {
        schema_version: ICRC_CAPABILITIES_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id.clone(),
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: ICRC_FETCHED_BY.to_string(),
        supported_standards: capabilities.supported_standards,
        capabilities: capabilities.capabilities,
    })
}
