//! Module: icrc::live::build
//!
//! Responsibility: build public ICRC reports from source data.
//! Does not own: Candid calls, source implementations, command parsing, or rendering.
//! Boundary: validates request-local values and projects raw source data into reports.

use super::{
    ICRC_ACCOUNT_TRANSACTIONS_REPORT_SCHEMA_VERSION, ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION,
    ICRC_ARCHIVES_REPORT_SCHEMA_VERSION, ICRC_BALANCE_REPORT_SCHEMA_VERSION,
    ICRC_BLOCK_TYPES_REPORT_SCHEMA_VERSION, ICRC_CAPABILITIES_REPORT_SCHEMA_VERSION,
    ICRC_FETCHED_BY, ICRC_INDEX_REPORT_SCHEMA_VERSION, ICRC_TIP_CERTIFICATE_REPORT_SCHEMA_VERSION,
    ICRC_TOKEN_REPORT_SCHEMA_VERSION, ICRC_TRANSACTIONS_REPORT_SCHEMA_VERSION,
    IcrcAccountTransactionsSource, IcrcSource, LiveIcrcSource,
};
use crate::{
    icrc::{
        ledger::principal_from_text,
        model::{
            IcrcAccountTransactionsError, IcrcAccountTransactionsReport,
            IcrcAccountTransactionsRequest, IcrcAllowanceReport, IcrcAllowanceRequest,
            IcrcArchivesReport, IcrcArchivesRequest, IcrcBalanceReport, IcrcBalanceRequest,
            IcrcBlockTypesReport, IcrcBlockTypesRequest, IcrcCapabilitiesReport,
            IcrcCapabilitiesRequest, IcrcError, IcrcIndexReport, IcrcIndexRequest,
            IcrcTipCertificateReport, IcrcTipCertificateRequest, IcrcTokenReport, IcrcTokenRequest,
            IcrcTransactionsReport, IcrcTransactionsRequest, normalize_subaccount_hex,
        },
    },
    subnet_catalog::format_utc_timestamp_secs,
};

pub fn build_icrc_token_report(request: &IcrcTokenRequest) -> Result<IcrcTokenReport, IcrcError> {
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
pub fn build_icrc_account_transactions_report(
    request: &IcrcAccountTransactionsRequest,
) -> Result<IcrcAccountTransactionsReport, IcrcAccountTransactionsError> {
    build_icrc_account_transactions_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_index_report(request: &IcrcIndexRequest) -> Result<IcrcIndexReport, IcrcError> {
    build_icrc_index_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_transactions_report(
    request: &IcrcTransactionsRequest,
) -> Result<IcrcTransactionsReport, IcrcError> {
    build_icrc_transactions_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_block_types_report(
    request: &IcrcBlockTypesRequest,
) -> Result<IcrcBlockTypesReport, IcrcError> {
    build_icrc_block_types_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_archives_report(
    request: &IcrcArchivesRequest,
) -> Result<IcrcArchivesReport, IcrcError> {
    build_icrc_archives_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_tip_certificate_report(
    request: &IcrcTipCertificateRequest,
) -> Result<IcrcTipCertificateReport, IcrcError> {
    build_icrc_tip_certificate_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_capabilities_report(
    request: &IcrcCapabilitiesRequest,
) -> Result<IcrcCapabilitiesReport, IcrcError> {
    build_icrc_capabilities_report_with_source(request, &LiveIcrcSource)
}

pub fn build_icrc_token_report_with_source(
    request: &IcrcTokenRequest,
    source: &dyn IcrcSource,
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
    source: &dyn IcrcSource,
) -> Result<IcrcBalanceReport, IcrcError> {
    let request = IcrcBalanceRequest {
        subaccount_hex: request
            .subaccount_hex
            .as_deref()
            .map(normalize_subaccount_hex)
            .transpose()?,
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
    source: &dyn IcrcSource,
) -> Result<IcrcAllowanceReport, IcrcError> {
    let request = IcrcAllowanceRequest {
        account_subaccount_hex: request
            .account_subaccount_hex
            .as_deref()
            .map(normalize_subaccount_hex)
            .transpose()?,
        spender_subaccount_hex: request
            .spender_subaccount_hex
            .as_deref()
            .map(normalize_subaccount_hex)
            .transpose()?,
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
pub fn build_icrc_account_transactions_report_with_source(
    request: &IcrcAccountTransactionsRequest,
    source: &dyn IcrcAccountTransactionsSource,
) -> Result<IcrcAccountTransactionsReport, IcrcAccountTransactionsError> {
    if request.limit == 0 {
        return Err(IcrcAccountTransactionsError::InvalidLimit {
            limit: request.limit,
        });
    }
    let request = IcrcAccountTransactionsRequest {
        subaccount_hex: request
            .subaccount_hex
            .as_deref()
            .map(normalize_subaccount_hex)
            .transpose()?,
        ..request.clone()
    };
    let transactions = source.fetch_account_transactions(&request)?;
    Ok(IcrcAccountTransactionsReport {
        schema_version: ICRC_ACCOUNT_TRANSACTIONS_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id,
        index_canister_id: transactions.index_canister_id,
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        requested_start: request.start.map(|start| start.to_string()),
        requested_limit: request.limit,
        next_start: transactions.next_start,
        oldest_transaction_id: transactions.oldest_transaction_id,
        balance: transactions.balance,
        token_symbol: transactions.token_symbol,
        decimals: transactions.decimals,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint,
        fetched_by: ICRC_FETCHED_BY.to_string(),
        transactions: transactions.transactions,
    })
}

pub fn build_icrc_index_report_with_source(
    request: &IcrcIndexRequest,
    source: &dyn IcrcSource,
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
    source: &dyn IcrcSource,
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
    request: &IcrcBlockTypesRequest,
    source: &dyn IcrcSource,
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
    source: &dyn IcrcSource,
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
    request: &IcrcTipCertificateRequest,
    source: &dyn IcrcSource,
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
    request: &IcrcCapabilitiesRequest,
    source: &dyn IcrcSource,
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
