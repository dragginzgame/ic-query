//! Module: icrc::live::fetch
//!
//! Responsibility: query live ICRC ledger methods and convert wire responses.
//! Does not own: source traits, synchronous runtime adaptation, report construction, or output.
//! Boundary: contains all generic ICRC host calls and wire-to-domain conversion.

use super::tip_certificate::verified_tip_certificate_data;
use crate::{
    hex::hex_bytes,
    icrc::{
        ledger::{
            GetIndexPrincipalResult, Icrc3ArchiveInfo, Icrc3ArchivedBlocks, Icrc3BlockWithId,
            Icrc3DataCertificate, Icrc3GetArchivesArgs, Icrc3GetBlocksRequest,
            Icrc3GetBlocksResult, Icrc3SupportedBlockType, Icrc3Value, IcrcAccount, IcrcAllowance,
            IcrcAllowanceArgs, IcrcLedgerMetadataRow, IcrcLedgerStandardRow,
            IcrcLedgerTokenMetadata, fetch_icrc_supported_standards, fetch_icrc1_token_metadata,
            ic_agent, index_principal_error_text, principal_from_text, query_ledger,
            query_ledger_arg,
        },
        model::{
            IcrcAllowanceData, IcrcAllowanceRequest, IcrcArchiveFollowErrorRow, IcrcArchiveRow,
            IcrcArchivedBlocksRow, IcrcArchivedRangeRow, IcrcArchivesData, IcrcArchivesRequest,
            IcrcBalanceData, IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesData,
            IcrcCapabilitiesData, IcrcCapabilityRow, IcrcError, IcrcFollowedArchiveBlockRow,
            IcrcIndexData, IcrcLedgerRequest, IcrcTipCertificateData, IcrcTokenData,
            IcrcTokenMetadataRow, IcrcTokenStandardRow, IcrcTransactionBlockRow,
            IcrcTransactionsData, IcrcTransactionsRequest, subaccount_bytes_from_hex,
        },
    },
};
use candid::{CandidType, Nat, Principal};
use ic_agent::Agent;
use serde_json::{Map as JsonMap, Value as JsonValue};

const ICRC1_SUPPORTED_STANDARDS_METHOD: &str = "icrc1_supported_standards";
const ICRC1_SYMBOL_METHOD: &str = "icrc1_symbol";
const ICRC1_DECIMALS_METHOD: &str = "icrc1_decimals";
const ICRC1_BALANCE_OF_METHOD: &str = "icrc1_balance_of";
const ICRC2_ALLOWANCE_METHOD: &str = "icrc2_allowance";
const ICRC106_GET_INDEX_PRINCIPAL_METHOD: &str = "icrc106_get_index_principal";
const ICRC3_GET_BLOCKS_METHOD: &str = "icrc3_get_blocks";
const ICRC3_SUPPORTED_BLOCK_TYPES_METHOD: &str = "icrc3_supported_block_types";
const ICRC3_GET_ARCHIVES_METHOD: &str = "icrc3_get_archives";
const ICRC3_GET_TIP_CERTIFICATE_METHOD: &str = "icrc3_get_tip_certificate";

pub(super) async fn fetch_token_async(
    request: &IcrcLedgerRequest,
) -> Result<IcrcTokenData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    Box::pin(fetch_icrc1_token_metadata::<IcrcError>(
        &agent,
        &ledger_canister,
    ))
    .await
    .map(token_data_from_ledger)
}

pub(super) async fn fetch_balance_async(
    request: &IcrcBalanceRequest,
) -> Result<IcrcBalanceData, IcrcError> {
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
        ICRC1_BALANCE_OF_METHOD,
        &account,
    )
    .await?;

    Ok(IcrcBalanceData {
        token_symbol,
        decimals,
        balance: balance.to_string(),
    })
}

pub(super) async fn fetch_allowance_async(
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
        ICRC2_ALLOWANCE_METHOD,
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

pub(super) async fn fetch_index_async(
    request: &IcrcLedgerRequest,
) -> Result<IcrcIndexData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let result = query_index_principal(&agent, &ledger_canister).await?;

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

pub(super) async fn fetch_transactions_async(
    request: &IcrcTransactionsRequest,
) -> Result<IcrcTransactionsData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let block_args = vec![Icrc3GetBlocksRequest {
        start: Nat::from(request.start),
        length: Nat::from(request.limit),
    }];
    let result = query_blocks(&agent, &ledger_canister, &block_args).await?;
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

        match query_blocks(agent, &archive.callback.0.principal, &archive.args).await {
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

pub(super) async fn fetch_block_types_async(
    request: &IcrcLedgerRequest,
) -> Result<IcrcBlockTypesData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let block_types = query_block_types(&agent, &ledger_canister).await?;

    Ok(IcrcBlockTypesData {
        block_types: block_types
            .into_iter()
            .map(block_type_row_from_wire)
            .collect(),
    })
}

pub(super) async fn fetch_archives_async(
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
    let archives = query_archives(&agent, &ledger_canister, &args).await?;

    Ok(IcrcArchivesData {
        archives: archives.into_iter().map(archive_row_from_wire).collect(),
    })
}

pub(super) async fn fetch_tip_certificate_async(
    request: &IcrcLedgerRequest,
) -> Result<IcrcTipCertificateData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    query_tip_certificate(&agent, &ledger_canister).await
}

pub(super) async fn fetch_capabilities_async(
    request: &IcrcLedgerRequest,
) -> Result<IcrcCapabilitiesData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let (standards_result, index, blocks, block_types, archives, tip_certificate) = futures::join!(
        fetch_icrc_supported_standards::<IcrcError>(&agent, &ledger_canister),
        fetch_index_capability(&agent, &ledger_canister),
        fetch_blocks_capability(&agent, &ledger_canister),
        fetch_block_types_capability(&agent, &ledger_canister),
        fetch_archives_capability(&agent, &ledger_canister),
        fetch_tip_certificate_capability(&agent, &ledger_canister),
    );
    let mut capabilities = Vec::with_capacity(6);

    let supported_standards = match standards_result {
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

    capabilities.extend([index, blocks, block_types, archives, tip_certificate]);

    Ok(IcrcCapabilitiesData {
        supported_standards,
        capabilities,
    })
}

async fn fetch_index_capability(agent: &Agent, ledger_canister: &Principal) -> IcrcCapabilityRow {
    match query_index_principal(agent, ledger_canister).await {
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
    match query_blocks(agent, ledger_canister, &block_args).await {
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
    match query_block_types(agent, ledger_canister).await {
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
    match query_archives(agent, ledger_canister, &args).await {
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
    match query_tip_certificate(agent, ledger_canister).await {
        Ok(IcrcTipCertificateData {
            certificate_bytes: Some(certificate_bytes),
            hash_tree_bytes: Some(hash_tree_bytes),
            ..
        }) => available_capability_row(
            "ICRC-3 tip certificate",
            ICRC3_GET_TIP_CERTIFICATE_METHOD,
            format!(
                "verified certificate {certificate_bytes} bytes, hash tree {hash_tree_bytes} bytes"
            ),
        ),
        Ok(_) => available_capability_row(
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

async fn query_blocks<Args>(
    agent: &Agent,
    canister: &Principal,
    args: &Args,
) -> Result<Icrc3GetBlocksResult, IcrcError>
where
    Args: CandidType + Sync,
{
    query_ledger_arg::<Args, Icrc3GetBlocksResult, IcrcError>(
        agent,
        canister,
        ICRC3_GET_BLOCKS_METHOD,
        args,
    )
    .await
}

async fn query_block_types(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<Vec<Icrc3SupportedBlockType>, IcrcError> {
    query_ledger::<Vec<Icrc3SupportedBlockType>, IcrcError>(
        agent,
        ledger_canister,
        ICRC3_SUPPORTED_BLOCK_TYPES_METHOD,
    )
    .await
}

async fn query_archives(
    agent: &Agent,
    ledger_canister: &Principal,
    args: &Icrc3GetArchivesArgs,
) -> Result<Vec<Icrc3ArchiveInfo>, IcrcError> {
    query_ledger_arg::<Icrc3GetArchivesArgs, Vec<Icrc3ArchiveInfo>, IcrcError>(
        agent,
        ledger_canister,
        ICRC3_GET_ARCHIVES_METHOD,
        args,
    )
    .await
}

async fn query_tip_certificate(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<IcrcTipCertificateData, IcrcError> {
    let certificate = query_ledger::<Option<Icrc3DataCertificate>, IcrcError>(
        agent,
        ledger_canister,
        ICRC3_GET_TIP_CERTIFICATE_METHOD,
    )
    .await?;
    verified_tip_certificate_data(agent, ledger_canister, certificate)
}

pub(super) async fn query_index_principal(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<GetIndexPrincipalResult, IcrcError> {
    query_ledger::<GetIndexPrincipalResult, IcrcError>(
        agent,
        ledger_canister,
        ICRC106_GET_INDEX_PRINCIPAL_METHOD,
    )
    .await
}

pub(super) fn live_query_context(
    source_endpoint: &str,
    ledger_canister_id: &str,
) -> Result<(Agent, Principal), IcrcError> {
    Ok((
        ic_agent::<IcrcError>(source_endpoint)?,
        principal_from_text::<IcrcError>(ledger_canister_id, "ledger_canister_id")?,
    ))
}

pub(super) async fn query_token_display_fields(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<(String, u8), IcrcError> {
    let (token_symbol, decimals) = futures::try_join!(
        query_ledger::<String, IcrcError>(agent, ledger_canister, ICRC1_SYMBOL_METHOD),
        query_ledger::<u8, IcrcError>(agent, ledger_canister, ICRC1_DECIMALS_METHOD),
    )?;
    Ok((token_symbol, decimals))
}

pub(super) fn account_from_parts(
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
    let summary = block_summary_from_wire(block);
    IcrcTransactionBlockRow {
        index: summary.index,
        block_type: summary.block_type,
        transaction_kind: summary.transaction_kind,
        timestamp_unix_nanos: summary.timestamp_unix_nanos,
        amount_base_units: summary.amount_base_units,
        raw_block: summary.raw_block,
    }
}

fn archived_blocks_row_from_wire(archive: Icrc3ArchivedBlocks) -> IcrcArchivedBlocksRow {
    let Icrc3ArchivedBlocks { args, callback } = archive;
    IcrcArchivedBlocksRow {
        callback_canister_id: callback.0.principal.to_text(),
        callback_method: callback.0.method,
        ranges: archived_range_rows(&args),
    }
}

fn followed_archive_block_row_from_wire(
    archive_canister_id: &str,
    callback_method: &str,
    block: Icrc3BlockWithId,
) -> IcrcFollowedArchiveBlockRow {
    let summary = block_summary_from_wire(block);
    IcrcFollowedArchiveBlockRow {
        archive_canister_id: archive_canister_id.to_string(),
        callback_method: callback_method.to_string(),
        index: summary.index,
        block_type: summary.block_type,
        transaction_kind: summary.transaction_kind,
        timestamp_unix_nanos: summary.timestamp_unix_nanos,
        amount_base_units: summary.amount_base_units,
        raw_block: summary.raw_block,
    }
}

struct Icrc3BlockSummary {
    index: String,
    block_type: Option<String>,
    transaction_kind: Option<String>,
    timestamp_unix_nanos: Option<String>,
    amount_base_units: Option<String>,
    raw_block: JsonValue,
}

fn block_summary_from_wire(block: Icrc3BlockWithId) -> Icrc3BlockSummary {
    let block_type = icrc3_text_at_path(&block.block, &["btype"]);
    Icrc3BlockSummary {
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
        ranges: archived_range_rows(&archive.args),
        error,
    }
}

fn archived_range_rows(ranges: &[Icrc3GetBlocksRequest]) -> Vec<IcrcArchivedRangeRow> {
    ranges
        .iter()
        .map(|range| IcrcArchivedRangeRow {
            start: range.start.to_string(),
            length: range.length.to_string(),
        })
        .collect()
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
