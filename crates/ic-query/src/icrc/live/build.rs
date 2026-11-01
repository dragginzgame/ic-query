//! Module: icrc::live::build
//!
//! Responsibility: build public ICRC reports from source data.
//! Does not own: Candid calls, source implementations, command parsing, or rendering.
//! Boundary: validates request-local values and projects raw source data into reports.

use super::{
    ICRC_ACCOUNT_TRANSACTION_PAGE_REPORT_SCHEMA_VERSION, ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION,
    ICRC_ARCHIVES_REPORT_SCHEMA_VERSION, ICRC_BALANCE_REPORT_SCHEMA_VERSION,
    ICRC_BLOCK_TYPES_REPORT_SCHEMA_VERSION, ICRC_CAPABILITIES_REPORT_SCHEMA_VERSION,
    ICRC_INDEX_REPORT_SCHEMA_VERSION, ICRC_TIP_CERTIFICATE_REPORT_SCHEMA_VERSION,
    ICRC_TOKEN_REPORT_SCHEMA_VERSION, ICRC_TRANSACTIONS_REPORT_SCHEMA_VERSION,
    account_transactions::{
        normalize_account_transaction_identity, normalize_optional_index_canister_id,
        normalize_transaction_cursor, validate_account_transaction_page,
        validate_account_transaction_page_size,
    },
    source::{
        IcrcAccountTransactionPageSource, IcrcAllowanceSource, IcrcArchivesSource,
        IcrcBalanceSource, IcrcBlockTypesSource, IcrcCapabilitiesSource, IcrcIndexSource,
        IcrcTipCertificateSource, IcrcTokenSource, IcrcTransactionsSource, LiveIcrcSource,
    },
};
use crate::icrc::{
    ledger::principal_from_text,
    model::{
        IcrcAccountTransactionError, IcrcAccountTransactionPageReport,
        IcrcAccountTransactionPageRequest, IcrcAllowanceReport, IcrcAllowanceRequest,
        IcrcArchivesReport, IcrcArchivesRequest, IcrcBalanceReport, IcrcBalanceRequest,
        IcrcBlockTypesReport, IcrcCapabilitiesReport, IcrcError, IcrcIndexReport,
        IcrcLedgerRequest, IcrcTipCertificateReport, IcrcTokenReport, IcrcTransactionsReport,
        IcrcTransactionsRequest, normalize_optional_subaccount_hex,
    },
};

pub async fn build_icrc_token_report(
    request: &IcrcLedgerRequest,
) -> Result<IcrcTokenReport, IcrcError> {
    build_icrc_token_report_with_source(request, &LiveIcrcSource::default()).await
}

pub async fn build_icrc_balance_report(
    request: &IcrcBalanceRequest,
) -> Result<IcrcBalanceReport, IcrcError> {
    build_icrc_balance_report_with_source(request, &LiveIcrcSource::default()).await
}

pub async fn build_icrc_allowance_report(
    request: &IcrcAllowanceRequest,
) -> Result<IcrcAllowanceReport, IcrcError> {
    build_icrc_allowance_report_with_source(request, &LiveIcrcSource::default()).await
}

/// Resolves an ICRC index and builds one account-transaction page.
pub async fn build_icrc_account_transaction_page_report(
    request: &IcrcAccountTransactionPageRequest,
) -> Result<IcrcAccountTransactionPageReport, IcrcAccountTransactionError> {
    build_icrc_account_transaction_page_report_with_source(request, &LiveIcrcSource::default())
        .await
}

pub async fn build_icrc_index_report(
    request: &IcrcLedgerRequest,
) -> Result<IcrcIndexReport, IcrcError> {
    build_icrc_index_report_with_source(request, &LiveIcrcSource::default()).await
}

pub async fn build_icrc_transactions_report(
    request: &IcrcTransactionsRequest,
) -> Result<IcrcTransactionsReport, IcrcError> {
    build_icrc_transactions_report_with_source(request, &LiveIcrcSource::default()).await
}

pub async fn build_icrc_block_types_report(
    request: &IcrcLedgerRequest,
) -> Result<IcrcBlockTypesReport, IcrcError> {
    build_icrc_block_types_report_with_source(request, &LiveIcrcSource::default()).await
}

pub async fn build_icrc_archives_report(
    request: &IcrcArchivesRequest,
) -> Result<IcrcArchivesReport, IcrcError> {
    build_icrc_archives_report_with_source(request, &LiveIcrcSource::default()).await
}

pub async fn build_icrc_tip_certificate_report(
    request: &IcrcLedgerRequest,
) -> Result<IcrcTipCertificateReport, IcrcError> {
    build_icrc_tip_certificate_report_with_source(request, &LiveIcrcSource::default()).await
}

pub async fn build_icrc_capabilities_report(
    request: &IcrcLedgerRequest,
) -> Result<IcrcCapabilitiesReport, IcrcError> {
    build_icrc_capabilities_report_with_source(request, &LiveIcrcSource::default()).await
}

pub async fn build_icrc_token_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcTokenSource,
) -> Result<IcrcTokenReport, IcrcError> {
    let token = source.fetch_token(request).await?;
    Ok(IcrcTokenReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            request,
            ICRC_TOKEN_REPORT_SCHEMA_VERSION,
        ),
        token,
    })
}

pub async fn build_icrc_balance_report_with_source(
    request: &IcrcBalanceRequest,
    source: &dyn IcrcBalanceSource,
) -> Result<IcrcBalanceReport, IcrcError> {
    let request = IcrcBalanceRequest {
        subaccount_hex: normalize_optional_subaccount_hex(request.subaccount_hex.as_deref())?,
        ..request.clone()
    };
    let balance = source.fetch_balance(&request).await?;
    Ok(IcrcBalanceReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            &request.ledger,
            ICRC_BALANCE_REPORT_SCHEMA_VERSION,
        ),
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        value: balance,
    })
}

pub async fn build_icrc_allowance_report_with_source(
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
    let allowance = source.fetch_allowance(&request).await?;
    Ok(IcrcAllowanceReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            &request.ledger,
            ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION,
        ),
        account_owner: request.account_owner,
        account_subaccount_hex: request.account_subaccount_hex,
        spender_owner: request.spender_owner,
        spender_subaccount_hex: request.spender_subaccount_hex,
        value: allowance,
    })
}

/// Builds one account-transaction page from a caller-supplied index capability.
pub async fn build_icrc_account_transaction_page_report_with_source(
    request: &IcrcAccountTransactionPageRequest,
    source: &dyn IcrcAccountTransactionPageSource,
) -> Result<IcrcAccountTransactionPageReport, IcrcAccountTransactionError> {
    validate_account_transaction_page_size(request.limit)?;
    let (ledger_canister_id, account_owner) = normalize_account_transaction_identity(
        &request.ledger.ledger_canister_id,
        &request.account_owner,
    )?;
    let index_canister_id =
        normalize_optional_index_canister_id(request.index_canister_id.as_deref())?;
    let request = IcrcAccountTransactionPageRequest {
        ledger: IcrcLedgerRequest {
            ledger_canister_id,
            ..request.ledger.clone()
        },
        index_canister_id,
        account_owner,
        subaccount_hex: normalize_optional_subaccount_hex(request.subaccount_hex.as_deref())?,
        start: request
            .start
            .as_deref()
            .map(normalize_transaction_cursor)
            .transpose()?,
        limit: request.limit,
    };
    let transactions = source.fetch_account_transaction_page(&request).await?;
    let actual_index =
        principal_from_text(&transactions.index_canister_id, "index_canister_id")?.to_text();
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
    validate_account_transaction_page(
        &transactions.transactions,
        request.start.as_deref(),
        transactions.oldest_transaction_id.as_deref(),
        transactions.next_start.as_deref(),
    )?;
    Ok(IcrcAccountTransactionPageReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            &request.ledger,
            ICRC_ACCOUNT_TRANSACTION_PAGE_REPORT_SCHEMA_VERSION,
        ),
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        requested_start: request.start,
        requested_limit: request.limit,

        page: crate::icrc::IcrcAccountTransactionPageData {
            index_canister_id: actual_index,
            ..transactions
        },
    })
}

pub async fn build_icrc_index_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcIndexSource,
) -> Result<IcrcIndexReport, IcrcError> {
    let index = source.fetch_index(request).await?;
    Ok(IcrcIndexReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            request,
            ICRC_INDEX_REPORT_SCHEMA_VERSION,
        ),
        index,
    })
}

pub async fn build_icrc_transactions_report_with_source(
    request: &IcrcTransactionsRequest,
    source: &dyn IcrcTransactionsSource,
) -> Result<IcrcTransactionsReport, IcrcError> {
    let transactions = source.fetch_transactions(request).await?;
    super::fetch::validate_transactions_data(request, &transactions)?;
    Ok(IcrcTransactionsReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            &request.ledger,
            ICRC_TRANSACTIONS_REPORT_SCHEMA_VERSION,
        ),
        requested_start: request.start.to_string(),
        requested_limit: request.limit,
        follow_archives: request.follow_archives,
        transactions,
    })
}

pub async fn build_icrc_block_types_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcBlockTypesSource,
) -> Result<IcrcBlockTypesReport, IcrcError> {
    let block_types = source.fetch_block_types(request).await?;
    Ok(IcrcBlockTypesReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            request,
            ICRC_BLOCK_TYPES_REPORT_SCHEMA_VERSION,
        ),
        block_types,
    })
}

pub async fn build_icrc_archives_report_with_source(
    request: &IcrcArchivesRequest,
    source: &dyn IcrcArchivesSource,
) -> Result<IcrcArchivesReport, IcrcError> {
    let request = IcrcArchivesRequest {
        from_canister_id: request
            .from_canister_id
            .as_deref()
            .map(|canister_id| {
                principal_from_text(canister_id, "from_canister_id")
                    .map(|principal| principal.to_text())
            })
            .transpose()?,
        ..request.clone()
    };
    let archives = source.fetch_archives(&request).await?;
    Ok(IcrcArchivesReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            &request.ledger,
            ICRC_ARCHIVES_REPORT_SCHEMA_VERSION,
        ),
        from_canister_id: request.from_canister_id,
        archives,
    })
}

pub async fn build_icrc_tip_certificate_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcTipCertificateSource,
) -> Result<IcrcTipCertificateReport, IcrcError> {
    let certificate = source.fetch_tip_certificate(request).await?;
    Ok(IcrcTipCertificateReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            request,
            ICRC_TIP_CERTIFICATE_REPORT_SCHEMA_VERSION,
        ),
        certificate: certificate.map(|evidence| crate::icrc::IcrcTipCertificateEvidence {
            certificate_hex: crate::hex::hex_bytes(&evidence.certificate),
            certificate_bytes: evidence.certificate.len(),
            hash_tree_hex: crate::hex::hex_bytes(&evidence.hash_tree),
            hash_tree_bytes: evidence.hash_tree.len(),
        }),
    })
}

pub async fn build_icrc_capabilities_report_with_source(
    request: &IcrcLedgerRequest,
    source: &dyn IcrcCapabilitiesSource,
) -> Result<IcrcCapabilitiesReport, IcrcError> {
    let capabilities = source.fetch_capabilities(request).await?;
    Ok(IcrcCapabilitiesReport {
        provenance: crate::icrc::IcrcReportProvenance::from_request(
            request,
            ICRC_CAPABILITIES_REPORT_SCHEMA_VERSION,
        ),
        supported_standards: capabilities.supported_standards,
        capabilities: capabilities.capabilities,
    })
}
