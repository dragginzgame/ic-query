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
            Icrc3GetArchivesArgs, Icrc3GetBlocksRequest, Icrc3GetBlocksResult,
            Icrc3SupportedBlockType, Icrc3Value, IcrcAccount, IcrcAllowance, IcrcAllowanceArgs,
            IcrcLedgerError, IcrcLedgerMetadataRow, IcrcLedgerStandardRow, IcrcLedgerTokenMetadata,
            fetch_icrc1_token_metadata, ic_agent, index_principal_error_text, principal_from_text,
            query_ledger, query_ledger_arg,
        },
        model::{
            IcrcAllowanceData, IcrcAllowanceReport, IcrcAllowanceRequest, IcrcArchiveRow,
            IcrcArchivedBlocksRow, IcrcArchivedRangeRow, IcrcArchivesData, IcrcArchivesReport,
            IcrcArchivesRequest, IcrcBalanceData, IcrcBalanceReport, IcrcBalanceRequest,
            IcrcBlockTypeRow, IcrcBlockTypesData, IcrcBlockTypesReport, IcrcBlockTypesRequest,
            IcrcError, IcrcIndexData, IcrcIndexReport, IcrcIndexRequest, IcrcTokenData,
            IcrcTokenMetadataRow, IcrcTokenReport, IcrcTokenRequest, IcrcTokenStandardRow,
            IcrcTransactionBlockRow, IcrcTransactionsData, IcrcTransactionsReport,
            IcrcTransactionsRequest, normalize_subaccount_hex, subaccount_bytes_from_hex,
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
pub(in crate::icrc) const ICRC_FETCHED_BY: &str = "ic-query";

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
/// Source contract for fetching generic ICRC ledger metadata, balances, allowances, indexes, and transactions.
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
        log_length: transactions.log_length,
        blocks: transactions.blocks,
        archived_blocks: transactions.archived_blocks,
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

    Ok(transactions_data_from_blocks(result))
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

fn transactions_data_from_blocks(result: Icrc3GetBlocksResult) -> IcrcTransactionsData {
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
