//! Module: icrc::live::fetch::history
//!
//! Responsibility: query and project ICRC index, block, archive, and tip evidence.
//! Does not own: account point values, capability policy, source traits, caching, or rendering.
//! Boundary: owns ICRC-3 history traversal and wire-to-report conversion.

use super::{super::tip_certificate::verified_tip_certificate_data, live_query_context};
use crate::{
    hex::hex_bytes,
    icrc::{
        ledger::{
            GetIndexPrincipalResult, Icrc3ArchiveInfo, Icrc3ArchivedBlocks, Icrc3BlockWithId,
            Icrc3DataCertificate, Icrc3GetArchivesArgs, Icrc3GetBlocksRequest,
            Icrc3GetBlocksResult, Icrc3SupportedBlockType, Icrc3Value, index_principal_error_text,
            principal_from_text, query_ledger, query_ledger_arg,
        },
        model::{
            IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow,
            IcrcArchivesData, IcrcArchivesRequest, IcrcBlockTypeRow, IcrcBlockTypesData, IcrcError,
            IcrcFollowedArchiveBlockRow, IcrcIndexData, IcrcLedgerRequest, IcrcTipCertificateData,
            IcrcTransactionBlockRow, IcrcTransactionsData, IcrcTransactionsRequest,
        },
    },
};
use candid::{CandidType, Nat, Principal};
use ic_agent::Agent;
use serde_json::{Map as JsonMap, Value as JsonValue};

pub(super) const ICRC106_GET_INDEX_PRINCIPAL_METHOD: &str = "icrc106_get_index_principal";
pub(super) const ICRC3_GET_BLOCKS_METHOD: &str = "icrc3_get_blocks";
pub(super) const ICRC3_SUPPORTED_BLOCK_TYPES_METHOD: &str = "icrc3_supported_block_types";
pub(super) const ICRC3_GET_ARCHIVES_METHOD: &str = "icrc3_get_archives";
pub(super) const ICRC3_GET_TIP_CERTIFICATE_METHOD: &str = "icrc3_get_tip_certificate";

pub(in crate::icrc::live) async fn fetch_index_async(
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

pub(in crate::icrc::live) async fn fetch_transactions_async(
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

pub(in crate::icrc::live) async fn fetch_block_types_async(
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

pub(in crate::icrc::live) async fn fetch_archives_async(
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

pub(in crate::icrc::live) async fn fetch_tip_certificate_async(
    request: &IcrcLedgerRequest,
) -> Result<IcrcTipCertificateData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    query_tip_certificate(&agent, &ledger_canister).await
}

pub(super) async fn query_blocks<Args>(
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

pub(super) async fn query_block_types(
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

pub(super) async fn query_archives(
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

pub(super) async fn query_tip_certificate(
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

pub(in crate::icrc::live) async fn query_index_principal(
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
