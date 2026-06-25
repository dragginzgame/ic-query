//! Module: icrc::live
//!
//! Responsibility: build ICRC reports and query live generic ICRC ledgers.
//! Does not own: command parsing, text rendering, or cache behavior.
//! Boundary: keeps live Candid calls behind a source trait for fixture tests.

use crate::{
    hex::hex_bytes,
    icrc::{
        ledger::{
            GetIndexPrincipalResult, Icrc3ArchiveInfo, Icrc3ArchivedBlocks, Icrc3BlockWithId,
            Icrc3DataCertificate, Icrc3GetArchivesArgs, Icrc3GetBlocksRequest,
            Icrc3GetBlocksResult, Icrc3SupportedBlockType, Icrc3Value, IcrcAccount, IcrcAllowance,
            IcrcAllowanceArgs, IcrcLedgerError, IcrcLedgerMetadataRow, IcrcLedgerStandardRow,
            IcrcLedgerTokenMetadata, fetch_icrc_supported_standards, fetch_icrc1_token_metadata,
            ic_agent, index_principal_error_text, principal_from_text, query_ledger,
            query_ledger_arg,
        },
        model::{
            IcrcAllowanceData, IcrcAllowanceReport, IcrcAllowanceRequest,
            IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow,
            IcrcArchivesData, IcrcArchivesReport, IcrcArchivesRequest, IcrcBalanceData,
            IcrcBalanceReport, IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesData,
            IcrcBlockTypesReport, IcrcBlockTypesRequest, IcrcCapabilitiesData,
            IcrcCapabilitiesReport, IcrcCapabilitiesRequest, IcrcCapabilityRow, IcrcError,
            IcrcFollowedArchiveBlockRow, IcrcIndexData, IcrcIndexReport, IcrcIndexRequest,
            IcrcTipCertificateData, IcrcTipCertificateReport, IcrcTipCertificateRequest,
            IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenReport, IcrcTokenRequest,
            IcrcTokenStandardRow, IcrcTransactionBlockRow, IcrcTransactionsData,
            IcrcTransactionsReport, IcrcTransactionsRequest, normalize_subaccount_hex,
            subaccount_bytes_from_hex,
        },
    },
    runtime::block_on_current_thread,
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::{Nat, Principal};
use ic_agent::Agent;
use serde_json::{Map as JsonMap, Value as JsonValue};

pub(in crate::icrc) const ICRC_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BALANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_INDEX_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_TRANSACTIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BLOCK_TYPES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ARCHIVES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_TIP_CERTIFICATE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_CAPABILITIES_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_FETCHED_BY: &str = "ic-query";
const ICRC1_SUPPORTED_STANDARDS_METHOD: &str = "icrc1_supported_standards";
const ICRC106_GET_INDEX_PRINCIPAL_METHOD: &str = "icrc106_get_index_principal";
const ICRC3_GET_BLOCKS_METHOD: &str = "icrc3_get_blocks";
const ICRC3_SUPPORTED_BLOCK_TYPES_METHOD: &str = "icrc3_supported_block_types";
const ICRC3_GET_ARCHIVES_METHOD: &str = "icrc3_get_archives";
const ICRC3_GET_TIP_CERTIFICATE_METHOD: &str = "icrc3_get_tip_certificate";

impl IcrcLedgerError for IcrcError {
    fn agent_build(endpoint: &str, reason: String) -> Self {
        Self::AgentBuild {
            endpoint: endpoint.to_string(),
            reason,
        }
    }

    fn invalid_principal(field: &'static str, reason: String) -> Self {
        Self::InvalidPrincipal { field, reason }
    }

    fn candid_encode(message: &'static str, reason: String) -> Self {
        Self::CandidEncode { message, reason }
    }

    fn agent_call(method: &'static str, reason: String) -> Self {
        Self::AgentCall { method, reason }
    }

    fn candid_decode(message: &'static str, reason: String) -> Self {
        Self::CandidDecode { message, reason }
    }
}

///
/// IcrcSource
///
/// Source contract for fetching generic ICRC ledger metadata, balances, allowances, indexes, and ICRC-3 rows.
///
pub(in crate::icrc) trait IcrcSource {
    fn fetch_token(&self, request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError>;

    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError>;

    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError>;

    fn fetch_index(&self, request: &IcrcIndexRequest) -> Result<IcrcIndexData, IcrcError>;

    fn fetch_transactions(
        &self,
        request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError>;

    fn fetch_block_types(
        &self,
        request: &IcrcBlockTypesRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError>;

    fn fetch_archives(&self, request: &IcrcArchivesRequest) -> Result<IcrcArchivesData, IcrcError>;

    fn fetch_tip_certificate(
        &self,
        request: &IcrcTipCertificateRequest,
    ) -> Result<IcrcTipCertificateData, IcrcError>;

    fn fetch_capabilities(
        &self,
        request: &IcrcCapabilitiesRequest,
    ) -> Result<IcrcCapabilitiesData, IcrcError>;
}

///
/// LiveIcrcSource
///
/// Source implementation backed by live ICRC ledger canister queries.
///
pub(in crate::icrc) struct LiveIcrcSource;

impl IcrcSource for LiveIcrcSource {
    fn fetch_token(&self, request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError> {
        block_on_current_thread(fetch_token_async(request))?
    }

    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
        block_on_current_thread(fetch_balance_async(request))?
    }

    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError> {
        block_on_current_thread(fetch_allowance_async(request))?
    }

    fn fetch_index(&self, request: &IcrcIndexRequest) -> Result<IcrcIndexData, IcrcError> {
        block_on_current_thread(fetch_index_async(request))?
    }

    fn fetch_transactions(
        &self,
        request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError> {
        block_on_current_thread(fetch_transactions_async(request))?
    }

    fn fetch_block_types(
        &self,
        request: &IcrcBlockTypesRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError> {
        block_on_current_thread(fetch_block_types_async(request))?
    }

    fn fetch_archives(&self, request: &IcrcArchivesRequest) -> Result<IcrcArchivesData, IcrcError> {
        block_on_current_thread(fetch_archives_async(request))?
    }

    fn fetch_tip_certificate(
        &self,
        request: &IcrcTipCertificateRequest,
    ) -> Result<IcrcTipCertificateData, IcrcError> {
        block_on_current_thread(fetch_tip_certificate_async(request))?
    }

    fn fetch_capabilities(
        &self,
        request: &IcrcCapabilitiesRequest,
    ) -> Result<IcrcCapabilitiesData, IcrcError> {
        block_on_current_thread(fetch_capabilities_async(request))?
    }
}

pub(in crate::icrc) fn build_icrc_token_report(
    request: &IcrcTokenRequest,
) -> Result<IcrcTokenReport, IcrcError> {
    build_icrc_token_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_balance_report(
    request: &IcrcBalanceRequest,
) -> Result<IcrcBalanceReport, IcrcError> {
    build_icrc_balance_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_allowance_report(
    request: &IcrcAllowanceRequest,
) -> Result<IcrcAllowanceReport, IcrcError> {
    build_icrc_allowance_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_index_report(
    request: &IcrcIndexRequest,
) -> Result<IcrcIndexReport, IcrcError> {
    build_icrc_index_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_transactions_report(
    request: &IcrcTransactionsRequest,
) -> Result<IcrcTransactionsReport, IcrcError> {
    build_icrc_transactions_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_block_types_report(
    request: &IcrcBlockTypesRequest,
) -> Result<IcrcBlockTypesReport, IcrcError> {
    build_icrc_block_types_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_archives_report(
    request: &IcrcArchivesRequest,
) -> Result<IcrcArchivesReport, IcrcError> {
    build_icrc_archives_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_tip_certificate_report(
    request: &IcrcTipCertificateRequest,
) -> Result<IcrcTipCertificateReport, IcrcError> {
    build_icrc_tip_certificate_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_capabilities_report(
    request: &IcrcCapabilitiesRequest,
) -> Result<IcrcCapabilitiesReport, IcrcError> {
    build_icrc_capabilities_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_token_report_with_source(
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

pub(in crate::icrc) fn build_icrc_balance_report_with_source(
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

pub(in crate::icrc) fn build_icrc_allowance_report_with_source(
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

pub(in crate::icrc) fn build_icrc_index_report_with_source(
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

pub(in crate::icrc) fn build_icrc_transactions_report_with_source(
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

pub(in crate::icrc) fn build_icrc_block_types_report_with_source(
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

pub(in crate::icrc) fn build_icrc_archives_report_with_source(
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

pub(in crate::icrc) fn build_icrc_tip_certificate_report_with_source(
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

pub(in crate::icrc) fn build_icrc_capabilities_report_with_source(
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

async fn fetch_token_async(request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    fetch_icrc1_token_metadata::<IcrcError>(&agent, &ledger_canister)
        .await
        .map(token_data_from_ledger)
}

async fn fetch_balance_async(request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let account = account_from_parts(
        &request.account_owner,
        request.subaccount_hex.as_deref(),
        "account_owner",
    )?;
    let (token_symbol, decimals) = query_token_display_fields(&agent, &ledger_canister).await?;
    let balance: Nat = query_ledger_arg::<IcrcAccount, Nat, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc1_balance_of",
        &account,
    )
    .await?;

    Ok(IcrcBalanceData {
        token_symbol,
        decimals,
        balance: balance.to_string(),
    })
}

async fn fetch_allowance_async(
    request: &IcrcAllowanceRequest,
) -> Result<IcrcAllowanceData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let allowance_args = IcrcAllowanceArgs {
        account: account_from_parts(
            &request.account_owner,
            request.account_subaccount_hex.as_deref(),
            "account_owner",
        )?,
        spender: account_from_parts(
            &request.spender_owner,
            request.spender_subaccount_hex.as_deref(),
            "spender_owner",
        )?,
    };
    let (token_symbol, decimals) = query_token_display_fields(&agent, &ledger_canister).await?;
    let allowance = query_ledger_arg::<IcrcAllowanceArgs, IcrcAllowance, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc2_allowance",
        &allowance_args,
    )
    .await?;

    Ok(IcrcAllowanceData {
        token_symbol,
        decimals,
        allowance: allowance.allowance.to_string(),
        expires_at_unix_nanos: allowance
            .expires_at
            .map(|expires_at| expires_at.to_string()),
    })
}

async fn fetch_index_async(request: &IcrcIndexRequest) -> Result<IcrcIndexData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let result = query_ledger::<GetIndexPrincipalResult, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc106_get_index_principal",
    )
    .await?;

    Ok(match result {
        GetIndexPrincipalResult::Ok(principal) => IcrcIndexData {
            index_canister_id: Some(principal.to_text()),
            index_error: None,
        },
        GetIndexPrincipalResult::Err(error) => IcrcIndexData {
            index_canister_id: None,
            index_error: Some(index_principal_error_text(error)),
        },
    })
}

async fn fetch_transactions_async(
    request: &IcrcTransactionsRequest,
) -> Result<IcrcTransactionsData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let block_args = vec![Icrc3GetBlocksRequest {
        start: Nat::from(request.start),
        length: Nat::from(request.limit),
    }];
    let result = query_ledger_arg::<Vec<Icrc3GetBlocksRequest>, Icrc3GetBlocksResult, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc3_get_blocks",
        &block_args,
    )
    .await?;
    let followed_archives = if request.follow_archives {
        fetch_archive_blocks(&agent, &result.archived_blocks).await
    } else {
        ArchiveFollowResult::default()
    };

    Ok(transactions_data_from_blocks(result, followed_archives))
}

#[derive(Default)]
struct ArchiveFollowResult {
    blocks: Vec<IcrcFollowedArchiveBlockRow>,
    errors: Vec<IcrcArchiveFollowErrorRow>,
}

async fn fetch_archive_blocks(
    agent: &Agent,
    archives: &[Icrc3ArchivedBlocks],
) -> ArchiveFollowResult {
    let mut result = ArchiveFollowResult::default();
    for archive in archives {
        let canister_id = archive.callback.0.principal.to_text();
        let method = archive.callback.0.method.clone();
        if method != ICRC3_GET_BLOCKS_METHOD {
            result.errors.push(archive_follow_error_row(
                archive,
                format!(
                    "unsupported archive callback method {method}; expected {ICRC3_GET_BLOCKS_METHOD}"
                ),
            ));
            continue;
        }

        match query_ledger_arg::<Vec<Icrc3GetBlocksRequest>, Icrc3GetBlocksResult, IcrcError>(
            agent,
            &archive.callback.0.principal,
            ICRC3_GET_BLOCKS_METHOD,
            &archive.args,
        )
        .await
        {
            Ok(blocks) => {
                result.blocks.extend(blocks.blocks.into_iter().map(|block| {
                    followed_archive_block_row_from_wire(&canister_id, &method, block)
                }));
            }
            Err(err) => result
                .errors
                .push(archive_follow_error_row(archive, err.to_string())),
        }
    }
    result
}

async fn fetch_block_types_async(
    request: &IcrcBlockTypesRequest,
) -> Result<IcrcBlockTypesData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let block_types = query_ledger::<Vec<Icrc3SupportedBlockType>, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc3_supported_block_types",
    )
    .await?;

    Ok(IcrcBlockTypesData {
        block_types: block_types
            .into_iter()
            .map(block_type_row_from_wire)
            .collect(),
    })
}

async fn fetch_archives_async(
    request: &IcrcArchivesRequest,
) -> Result<IcrcArchivesData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let args = Icrc3GetArchivesArgs {
        from: request
            .from_canister_id
            .as_deref()
            .map(|from| principal_from_text::<IcrcError>(from, "from_canister_id"))
            .transpose()?,
    };
    let archives = query_ledger_arg::<Icrc3GetArchivesArgs, Vec<Icrc3ArchiveInfo>, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc3_get_archives",
        &args,
    )
    .await?;

    Ok(IcrcArchivesData {
        archives: archives.into_iter().map(archive_row_from_wire).collect(),
    })
}

async fn fetch_tip_certificate_async(
    request: &IcrcTipCertificateRequest,
) -> Result<IcrcTipCertificateData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let certificate = query_ledger::<Option<Icrc3DataCertificate>, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc3_get_tip_certificate",
    )
    .await?;

    Ok(tip_certificate_data_from_wire(certificate))
}

async fn fetch_capabilities_async(
    request: &IcrcCapabilitiesRequest,
) -> Result<IcrcCapabilitiesData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let mut capabilities = Vec::new();

    let supported_standards =
        match fetch_icrc_supported_standards::<IcrcError>(&agent, &ledger_canister).await {
            Ok(standards) => {
                capabilities.push(available_capability_row(
                    "ICRC-1 supported standards",
                    ICRC1_SUPPORTED_STANDARDS_METHOD,
                    format!("{} standard(s)", standards.len()),
                ));
                standards
                    .into_iter()
                    .map(token_standard_row_from_ledger)
                    .collect()
            }
            Err(err) => {
                capabilities.push(error_capability_row(
                    "ICRC-1 supported standards",
                    ICRC1_SUPPORTED_STANDARDS_METHOD,
                    err,
                ));
                Vec::new()
            }
        };

    capabilities.push(fetch_index_capability(&agent, &ledger_canister).await);
    capabilities.push(fetch_blocks_capability(&agent, &ledger_canister).await);
    capabilities.push(fetch_block_types_capability(&agent, &ledger_canister).await);
    capabilities.push(fetch_archives_capability(&agent, &ledger_canister).await);
    capabilities.push(fetch_tip_certificate_capability(&agent, &ledger_canister).await);

    Ok(IcrcCapabilitiesData {
        supported_standards,
        capabilities,
    })
}

async fn fetch_index_capability(agent: &Agent, ledger_canister: &Principal) -> IcrcCapabilityRow {
    match query_ledger::<GetIndexPrincipalResult, IcrcError>(
        agent,
        ledger_canister,
        ICRC106_GET_INDEX_PRINCIPAL_METHOD,
    )
    .await
    {
        Ok(GetIndexPrincipalResult::Ok(principal)) => available_capability_row(
            "ICRC-106 index discovery",
            ICRC106_GET_INDEX_PRINCIPAL_METHOD,
            format!("index canister {}", principal.to_text()),
        ),
        Ok(GetIndexPrincipalResult::Err(error)) => available_capability_row(
            "ICRC-106 index discovery",
            ICRC106_GET_INDEX_PRINCIPAL_METHOD,
            index_principal_error_text(error),
        ),
        Err(err) => error_capability_row(
            "ICRC-106 index discovery",
            ICRC106_GET_INDEX_PRINCIPAL_METHOD,
            err,
        ),
    }
}

async fn fetch_blocks_capability(agent: &Agent, ledger_canister: &Principal) -> IcrcCapabilityRow {
    let block_args = vec![Icrc3GetBlocksRequest {
        start: Nat::from(0_u64),
        length: Nat::from(1_u64),
    }];
    match query_ledger_arg::<Vec<Icrc3GetBlocksRequest>, Icrc3GetBlocksResult, IcrcError>(
        agent,
        ledger_canister,
        ICRC3_GET_BLOCKS_METHOD,
        &block_args,
    )
    .await
    {
        Ok(result) => available_capability_row(
            "ICRC-3 block history",
            ICRC3_GET_BLOCKS_METHOD,
            format!(
                "log_length {}, returned_blocks {}, archive_callbacks {}",
                result.log_length,
                result.blocks.len(),
                result.archived_blocks.len()
            ),
        ),
        Err(err) => error_capability_row("ICRC-3 block history", ICRC3_GET_BLOCKS_METHOD, err),
    }
}

async fn fetch_block_types_capability(
    agent: &Agent,
    ledger_canister: &Principal,
) -> IcrcCapabilityRow {
    match query_ledger::<Vec<Icrc3SupportedBlockType>, IcrcError>(
        agent,
        ledger_canister,
        ICRC3_SUPPORTED_BLOCK_TYPES_METHOD,
    )
    .await
    {
        Ok(block_types) => available_capability_row(
            "ICRC-3 supported block types",
            ICRC3_SUPPORTED_BLOCK_TYPES_METHOD,
            named_count_detail(
                "block type",
                block_types
                    .iter()
                    .map(|block_type| block_type.block_type.as_str()),
            ),
        ),
        Err(err) => error_capability_row(
            "ICRC-3 supported block types",
            ICRC3_SUPPORTED_BLOCK_TYPES_METHOD,
            err,
        ),
    }
}

async fn fetch_archives_capability(
    agent: &Agent,
    ledger_canister: &Principal,
) -> IcrcCapabilityRow {
    let args = Icrc3GetArchivesArgs { from: None };
    match query_ledger_arg::<Icrc3GetArchivesArgs, Vec<Icrc3ArchiveInfo>, IcrcError>(
        agent,
        ledger_canister,
        ICRC3_GET_ARCHIVES_METHOD,
        &args,
    )
    .await
    {
        Ok(archives) => available_capability_row(
            "ICRC-3 archive discovery",
            ICRC3_GET_ARCHIVES_METHOD,
            format!("{} archive range(s)", archives.len()),
        ),
        Err(err) => {
            error_capability_row("ICRC-3 archive discovery", ICRC3_GET_ARCHIVES_METHOD, err)
        }
    }
}

async fn fetch_tip_certificate_capability(
    agent: &Agent,
    ledger_canister: &Principal,
) -> IcrcCapabilityRow {
    match query_ledger::<Option<Icrc3DataCertificate>, IcrcError>(
        agent,
        ledger_canister,
        ICRC3_GET_TIP_CERTIFICATE_METHOD,
    )
    .await
    {
        Ok(Some(certificate)) => available_capability_row(
            "ICRC-3 tip certificate",
            ICRC3_GET_TIP_CERTIFICATE_METHOD,
            format!(
                "certificate {} bytes, hash tree {} bytes",
                certificate.certificate.len(),
                certificate.hash_tree.len()
            ),
        ),
        Ok(None) => available_capability_row(
            "ICRC-3 tip certificate",
            ICRC3_GET_TIP_CERTIFICATE_METHOD,
            "certificate absent",
        ),
        Err(err) => error_capability_row(
            "ICRC-3 tip certificate",
            ICRC3_GET_TIP_CERTIFICATE_METHOD,
            err,
        ),
    }
}

fn live_query_context(
    source_endpoint: &str,
    ledger_canister_id: &str,
) -> Result<(Agent, Principal), IcrcError> {
    Ok((
        ic_agent::<IcrcError>(source_endpoint)?,
        principal_from_text::<IcrcError>(ledger_canister_id, "ledger_canister_id")?,
    ))
}

async fn query_token_display_fields(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<(String, u8), IcrcError> {
    let token_symbol =
        query_ledger::<String, IcrcError>(agent, ledger_canister, "icrc1_symbol").await?;
    let decimals = query_ledger::<u8, IcrcError>(agent, ledger_canister, "icrc1_decimals").await?;
    Ok((token_symbol, decimals))
}

fn account_from_parts(
    owner: &str,
    subaccount_hex: Option<&str>,
    owner_field: &'static str,
) -> Result<IcrcAccount, IcrcError> {
    Ok(IcrcAccount {
        owner: principal_from_text::<IcrcError>(owner, owner_field)?,
        subaccount: subaccount_hex.map(subaccount_bytes_from_hex).transpose()?,
    })
}

fn token_data_from_ledger(token: IcrcLedgerTokenMetadata) -> IcrcTokenData {
    IcrcTokenData {
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        supported_standards: token
            .supported_standards
            .into_iter()
            .map(token_standard_row_from_ledger)
            .collect(),
        metadata: token
            .metadata
            .into_iter()
            .map(token_metadata_row_from_ledger)
            .collect(),
    }
}

fn token_standard_row_from_ledger(row: IcrcLedgerStandardRow) -> IcrcTokenStandardRow {
    IcrcTokenStandardRow {
        name: row.name,
        url: row.url,
    }
}

fn token_metadata_row_from_ledger(row: IcrcLedgerMetadataRow) -> IcrcTokenMetadataRow {
    IcrcTokenMetadataRow {
        key: row.key,
        value_type: row.value_type,
        value: row.value,
    }
}

fn transactions_data_from_blocks(
    result: Icrc3GetBlocksResult,
    followed_archives: ArchiveFollowResult,
) -> IcrcTransactionsData {
    IcrcTransactionsData {
        log_length: Some(result.log_length.to_string()),
        blocks: result
            .blocks
            .into_iter()
            .map(transaction_block_row_from_wire)
            .collect(),
        archived_blocks: result
            .archived_blocks
            .into_iter()
            .map(archived_blocks_row_from_wire)
            .collect(),
        followed_archive_blocks: followed_archives.blocks,
        archive_follow_errors: followed_archives.errors,
    }
}

fn transaction_block_row_from_wire(block: Icrc3BlockWithId) -> IcrcTransactionBlockRow {
    let block_type = icrc3_text_at_path(&block.block, &["btype"]);
    IcrcTransactionBlockRow {
        index: block.id.to_string(),
        transaction_kind: block_type
            .clone()
            .or_else(|| icrc3_text_at_path(&block.block, &["tx", "op"])),
        block_type,
        timestamp_unix_nanos: icrc3_nat_at_path(&block.block, &["ts"]),
        amount_base_units: icrc3_nat_at_path(&block.block, &["tx", "amt"]),
        raw_block: icrc3_value_json(&block.block),
    }
}

fn archived_blocks_row_from_wire(archive: Icrc3ArchivedBlocks) -> IcrcArchivedBlocksRow {
    IcrcArchivedBlocksRow {
        callback_canister_id: archive.callback.0.principal.to_text(),
        callback_method: archive.callback.0.method,
        ranges: archive
            .args
            .into_iter()
            .map(|range| IcrcArchivedRangeRow {
                start: range.start.to_string(),
                length: range.length.to_string(),
            })
            .collect(),
    }
}

fn followed_archive_block_row_from_wire(
    archive_canister_id: &str,
    callback_method: &str,
    block: Icrc3BlockWithId,
) -> IcrcFollowedArchiveBlockRow {
    let block_type = icrc3_text_at_path(&block.block, &["btype"]);
    IcrcFollowedArchiveBlockRow {
        archive_canister_id: archive_canister_id.to_string(),
        callback_method: callback_method.to_string(),
        index: block.id.to_string(),
        transaction_kind: block_type
            .clone()
            .or_else(|| icrc3_text_at_path(&block.block, &["tx", "op"])),
        block_type,
        timestamp_unix_nanos: icrc3_nat_at_path(&block.block, &["ts"]),
        amount_base_units: icrc3_nat_at_path(&block.block, &["tx", "amt"]),
        raw_block: icrc3_value_json(&block.block),
    }
}

fn archive_follow_error_row(
    archive: &Icrc3ArchivedBlocks,
    error: String,
) -> IcrcArchiveFollowErrorRow {
    IcrcArchiveFollowErrorRow {
        callback_canister_id: archive.callback.0.principal.to_text(),
        callback_method: archive.callback.0.method.clone(),
        ranges: archive
            .args
            .iter()
            .map(|range| IcrcArchivedRangeRow {
                start: range.start.to_string(),
                length: range.length.to_string(),
            })
            .collect(),
        error,
    }
}

fn block_type_row_from_wire(block_type: Icrc3SupportedBlockType) -> IcrcBlockTypeRow {
    IcrcBlockTypeRow {
        block_type: block_type.block_type,
        url: block_type.url,
    }
}

fn archive_row_from_wire(archive: Icrc3ArchiveInfo) -> IcrcArchiveRow {
    IcrcArchiveRow {
        canister_id: archive.canister_id.to_text(),
        start: archive.start.to_string(),
        end: archive.end.to_string(),
    }
}

fn tip_certificate_data_from_wire(
    certificate: Option<Icrc3DataCertificate>,
) -> IcrcTipCertificateData {
    certificate.map_or(
        IcrcTipCertificateData {
            certificate_hex: None,
            certificate_bytes: None,
            hash_tree_hex: None,
            hash_tree_bytes: None,
        },
        |certificate| IcrcTipCertificateData {
            certificate_hex: Some(hex_bytes(&certificate.certificate)),
            certificate_bytes: Some(certificate.certificate.len()),
            hash_tree_hex: Some(hex_bytes(&certificate.hash_tree)),
            hash_tree_bytes: Some(certificate.hash_tree.len()),
        },
    )
}

fn available_capability_row(
    capability: &str,
    method: &'static str,
    details: impl Into<String>,
) -> IcrcCapabilityRow {
    IcrcCapabilityRow {
        capability: capability.to_string(),
        method: method.to_string(),
        status: "available".to_string(),
        details: Some(details.into()),
        error: None,
    }
}

fn error_capability_row(
    capability: &str,
    method: &'static str,
    error: IcrcError,
) -> IcrcCapabilityRow {
    let error = error.to_string();
    let status = capability_error_status(&error);
    IcrcCapabilityRow {
        capability: capability.to_string(),
        method: method.to_string(),
        status: status.to_string(),
        details: Some(capability_error_details(status).to_string()),
        error: Some(error),
    }
}

fn capability_error_status(error: &str) -> &'static str {
    if method_not_exported(error) {
        "unsupported"
    } else {
        "error"
    }
}

fn capability_error_details(status: &str) -> &'static str {
    if status == "unsupported" {
        "method not exported by target canister"
    } else {
        "query failed"
    }
}

fn method_not_exported(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("has no query method")
        || error.contains("method not found")
        || error.contains("ic0536")
}

fn named_count_detail<'a, I>(singular: &str, names: I) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    let names = names.into_iter().collect::<Vec<_>>();
    if names.is_empty() {
        format!("0 {singular}(s)")
    } else {
        format!("{} {singular}(s): {}", names.len(), names.join(", "))
    }
}

fn icrc3_text_at_path(value: &Icrc3Value, path: &[&str]) -> Option<String> {
    let value = icrc3_value_at_path(value, path)?;
    match value {
        Icrc3Value::Text(text) => Some(text.clone()),
        _ => None,
    }
}

fn icrc3_nat_at_path(value: &Icrc3Value, path: &[&str]) -> Option<String> {
    let value = icrc3_value_at_path(value, path)?;
    match value {
        Icrc3Value::Nat(nat) => Some(nat.to_string()),
        _ => None,
    }
}

fn icrc3_value_at_path<'a>(value: &'a Icrc3Value, path: &[&str]) -> Option<&'a Icrc3Value> {
    path.iter().try_fold(value, |value, key| match value {
        Icrc3Value::Map(map) => map.get(*key),
        _ => None,
    })
}

fn icrc3_value_json(value: &Icrc3Value) -> JsonValue {
    let mut variant = JsonMap::new();
    match value {
        Icrc3Value::Blob(bytes) => {
            variant.insert("Blob".to_string(), JsonValue::String(hex_bytes(bytes)));
        }
        Icrc3Value::Text(text) => {
            variant.insert("Text".to_string(), JsonValue::String(text.clone()));
        }
        Icrc3Value::Nat(nat) => {
            variant.insert("Nat".to_string(), JsonValue::String(nat.to_string()));
        }
        Icrc3Value::Int(int) => {
            variant.insert("Int".to_string(), JsonValue::String(int.to_string()));
        }
        Icrc3Value::Array(values) => {
            variant.insert(
                "Array".to_string(),
                JsonValue::Array(values.iter().map(icrc3_value_json).collect()),
            );
        }
        Icrc3Value::Map(entries) => {
            variant.insert(
                "Map".to_string(),
                JsonValue::Object(
                    entries
                        .iter()
                        .map(|(key, value)| (key.clone(), icrc3_value_json(value)))
                        .collect(),
                ),
            );
        }
    }
    JsonValue::Object(variant)
}
