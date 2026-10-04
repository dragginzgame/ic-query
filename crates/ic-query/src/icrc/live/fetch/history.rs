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
            nat_text, principal_from_text, query_ledger, query_ledger_arg,
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
use std::collections::BTreeSet;

const MAX_ARCHIVE_CALLBACKS: usize = 100;
const MAX_ARCHIVE_REPLY_BYTES: usize = 64 * 1024 * 1024;

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
    validate_block_page(&result, request)?;
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
    let mut remaining_bytes = MAX_ARCHIVE_REPLY_BYTES;
    for archive in archives {
        let canister_id = archive.callback.0.principal.to_text();
        let method = archive.callback.0.method.clone();
        match query_archive_blocks(agent, archive, &mut remaining_bytes).await {
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

async fn query_archive_blocks(
    agent: &Agent,
    archive: &Icrc3ArchivedBlocks,
    remaining_bytes: &mut usize,
) -> Result<Icrc3GetBlocksResult, IcrcError> {
    const CONTEXT: &str = "ICRC3 archive callback";
    if *remaining_bytes == 0 {
        return Err(invalid_transaction_page(
            "archive reply byte budget exhausted",
        ));
    }
    let arg = candid::encode_one(&archive.args).map_err(|error| IcrcError::CandidEncode {
        message: CONTEXT,
        reason: error.to_string(),
    })?;
    let bytes = agent
        .query(&archive.callback.0.principal, &archive.callback.0.method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|error| IcrcError::AgentCall {
            method: CONTEXT,
            reason: error.to_string(),
        })?;
    charge_archive_reply(remaining_bytes, bytes.len())?;
    let result: Icrc3GetBlocksResult =
        crate::candid_decode::decode_reply(&bytes).map_err(|error| IcrcError::CandidDecode {
            message: CONTEXT,
            reason: error.to_string(),
        })?;
    validate_archive_reply(&result, &archive.args)?;
    Ok(result)
}

fn invalid_transaction_page(reason: &str) -> IcrcError {
    IcrcError::InvalidTransactionPage {
        reason: reason.to_string(),
    }
}

fn charge_archive_reply(remaining: &mut usize, length: usize) -> Result<(), IcrcError> {
    let Some(next) = remaining.checked_sub(length) else {
        *remaining = 0;
        return Err(invalid_transaction_page(
            "archive reply byte budget exhausted",
        ));
    };
    *remaining = next;
    Ok(())
}

fn validate_block_page(
    result: &Icrc3GetBlocksResult,
    request: &IcrcTransactionsRequest,
) -> Result<(), IcrcError> {
    let mut ranges = Vec::new();
    for block in &result.blocks {
        ranges.push((block.id.clone(), Nat(&block.id.0 + 1u32)));
    }
    for archive in &result.archived_blocks {
        if archive.args.is_empty() {
            return Err(invalid_transaction_page("archive callback has no ranges"));
        }
        for range in &archive.args {
            ranges.push((range.start.clone(), Nat(&range.start.0 + &range.length.0)));
        }
    }
    validate_page_ranges(
        request,
        Some(&result.log_length),
        result.archived_blocks.len(),
        ranges,
    )
}

fn validate_page_ranges(
    request: &IcrcTransactionsRequest,
    log_length: Option<&Nat>,
    callback_count: usize,
    mut ranges: Vec<(Nat, Nat)>,
) -> Result<(), IcrcError> {
    if callback_count > MAX_ARCHIVE_CALLBACKS {
        return Err(invalid_transaction_page(
            "too many archive callbacks for one page",
        ));
    }
    let start = Nat::from(request.start);
    let end = Nat(&start.0 + Nat::from(request.limit).0);
    if ranges
        .iter()
        .any(|(from, to)| from < &start || to > &end || from >= to)
    {
        return Err(invalid_transaction_page(
            "block or archive range is outside the requested page or empty",
        ));
    }
    if log_length.is_some_and(|length| ranges.iter().any(|(_, to)| to > length)) {
        return Err(invalid_transaction_page(
            "block or archive range exceeds the ledger log length",
        ));
    }
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err(invalid_transaction_page(
            "ledger blocks and archive ranges overlap",
        ));
    }
    Ok(())
}

/// Validate the same page bounds at the public custom-source report boundary.
pub(in crate::icrc::live) fn validate_transactions_data(
    request: &IcrcTransactionsRequest,
    data: &IcrcTransactionsData,
) -> Result<(), IcrcError> {
    let parse = |text: &str| -> Result<Nat, IcrcError> {
        if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid_transaction_page(
                "log length, block indexes, and ranges must be decimal naturals",
            ));
        }
        text.parse()
            .map_err(|_| invalid_transaction_page("invalid decimal block index or range"))
    };
    let log_length = data.log_length.as_deref().map(parse).transpose()?;
    let mut ranges = Vec::new();
    let mut seen = BTreeSet::new();
    for block in &data.blocks {
        let id = parse(&block.index)?;
        ranges.push((id.clone(), Nat(&id.0 + 1u32)));
        seen.insert(id);
    }
    for archive in &data.archived_blocks {
        if archive.ranges.is_empty() {
            return Err(invalid_transaction_page("archive callback has no ranges"));
        }
        for range in &archive.ranges {
            let start = parse(&range.start)?;
            let length = parse(&range.length)?;
            ranges.push((start.clone(), Nat(start.0 + length.0)));
        }
    }
    validate_page_ranges(
        request,
        log_length.as_ref(),
        data.archived_blocks.len(),
        ranges,
    )?;
    if !request.follow_archives
        && (!data.followed_archive_blocks.is_empty() || !data.archive_follow_errors.is_empty())
    {
        return Err(invalid_transaction_page(
            "source followed archives without being requested",
        ));
    }
    for block in &data.followed_archive_blocks {
        let id = parse(&block.index)?;
        let mut in_range = false;
        for archive in &data.archived_blocks {
            if (
                archive.callback_canister_id.as_str(),
                archive.callback_method.as_str(),
            ) == (
                block.archive_canister_id.as_str(),
                block.callback_method.as_str(),
            ) {
                for range in &archive.ranges {
                    let start = parse(&range.start)?;
                    let end = Nat(&start.0 + parse(&range.length)?.0);
                    in_range |= id >= start && id < end;
                }
            }
        }
        if !in_range || !seen.insert(id) {
            return Err(invalid_transaction_page(
                "followed archive block is outside its callback ranges or duplicated",
            ));
        }
    }
    for error in &data.archive_follow_errors {
        if !data.archived_blocks.iter().any(|archive| {
            archive.callback_canister_id == error.callback_canister_id
                && archive.callback_method == error.callback_method
                && archive.ranges == error.ranges
        }) {
            return Err(invalid_transaction_page(
                "archive follow error does not match a returned callback",
            ));
        }
    }
    if data.archive_follow_errors.len() > data.archived_blocks.len() {
        return Err(invalid_transaction_page("too many archive follow errors"));
    }
    Ok(())
}

fn validate_archive_reply(
    result: &Icrc3GetBlocksResult,
    args: &[Icrc3GetBlocksRequest],
) -> Result<(), IcrcError> {
    let ranges: Vec<_> = args
        .iter()
        .map(|range| (&range.start, Nat(&range.start.0 + &range.length.0)))
        .collect();
    let mut ids = BTreeSet::new();
    for block in &result.blocks {
        if !ranges
            .iter()
            .any(|(start, end)| block.id >= **start && block.id < *end)
        {
            return Err(invalid_transaction_page(
                "archive block is outside its callback ranges",
            ));
        }
        if !ids.insert(&block.id) {
            return Err(invalid_transaction_page(
                "archive returned duplicate block ids",
            ));
        }
    }
    // Archive referrals are metadata only; following is deliberately one hop.
    Ok(())
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
        log_length: Some(nat_text(&result.log_length)),
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
        index: nat_text(&block.id),
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
            start: nat_text(&range.start),
            length: nat_text(&range.length),
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
        start: nat_text(&archive.start),
        end: nat_text(&archive.end),
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
        Icrc3Value::Nat(nat) => Some(nat_text(nat)),
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
            variant.insert("Nat".to_string(), JsonValue::String(nat_text(nat)));
        }
        Icrc3Value::Int(int) => {
            variant.insert("Int".to_string(), JsonValue::String(int.0.to_str_radix(10)));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{icrc::ledger::Icrc3ArchiveCallback, runtime::block_on_current_thread};
    use candid::Func;
    use serde_cbor::Value;
    use std::{
        collections::BTreeMap,
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
        thread,
    };

    #[test]
    fn wire_history_projects_plain_decimal_numbers_accepted_by_report_validation() {
        let large: Nat = "18446744073709551616".parse().unwrap();
        let block = Icrc3BlockWithId {
            id: Nat::from(1_000u32),
            block: Icrc3Value::Map(BTreeMap::from([
                ("ts".into(), Icrc3Value::Nat(large.clone())),
                (
                    "tx".into(),
                    Icrc3Value::Map(BTreeMap::from([(
                        "amt".into(),
                        Icrc3Value::Nat(large.clone()),
                    )])),
                ),
                ("signed".into(), Icrc3Value::Int(candid::Int::from(-1_000))),
            ])),
        };
        let result = Icrc3GetBlocksResult {
            log_length: large,
            blocks: vec![block.clone()],
            archived_blocks: vec![Icrc3ArchivedBlocks {
                args: vec![Icrc3GetBlocksRequest {
                    start: Nat::from(1_001u32),
                    length: Nat::from(1_000u32),
                }],
                callback: Icrc3ArchiveCallback(Func {
                    principal: Principal::anonymous(),
                    method: "read_page".into(),
                }),
            }],
        };
        let request = IcrcTransactionsRequest {
            source_endpoint: "fixture".into(),
            now_unix_secs: 0,
            ledger_canister_id: Principal::anonymous().to_text(),
            start: 1_000,
            limit: 1_001,
            follow_archives: true,
        };
        validate_block_page(&result, &request).unwrap();
        let mut followed = block;
        followed.id = Nat::from(1_001u32);
        let data = transactions_data_from_blocks(
            result,
            ArchiveFollowResult {
                blocks: vec![followed_archive_block_row_from_wire(
                    &Principal::anonymous().to_text(),
                    "read_page",
                    followed,
                )],
                errors: vec![],
            },
        );
        validate_transactions_data(&request, &data).expect("wire data satisfies report contract");
        assert_eq!(data.log_length.as_deref(), Some("18446744073709551616"));
        assert_eq!(data.blocks[0].index, "1000");
        assert_eq!(data.followed_archive_blocks[0].index, "1001");
        assert_eq!(data.archived_blocks[0].ranges[0].start, "1001");
        assert_eq!(data.archived_blocks[0].ranges[0].length, "1000");
        assert_eq!(
            data.blocks[0].amount_base_units.as_deref(),
            Some("18446744073709551616")
        );
        assert_eq!(
            data.blocks[0].timestamp_unix_nanos.as_deref(),
            Some("18446744073709551616")
        );
        assert_eq!(
            data.blocks[0].raw_block["Map"]["tx"]["Map"]["amt"]["Nat"],
            "18446744073709551616"
        );
        assert_eq!(data.blocks[0].raw_block["Map"]["signed"]["Int"], "-1000");
        let archive = archive_row_from_wire(Icrc3ArchiveInfo {
            canister_id: Principal::anonymous(),
            start: Nat::from(1_000u32),
            end: Nat::from(2_000u32),
        });
        assert_eq!(archive.start, "1000");
        assert_eq!(archive.end, "2000");
    }

    fn follow_callback_fixture(block_ids: &[u64]) -> ArchiveFollowResult {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let reply = Icrc3GetBlocksResult {
            log_length: Nat::from(100u64),
            blocks: block_ids
                .iter()
                .map(|id| Icrc3BlockWithId {
                    id: Nat::from(*id),
                    block: Icrc3Value::Text("fixture".into()),
                })
                .collect(),
            archived_blocks: vec![],
        };
        let body = serde_cbor::to_vec(&Value::Map(BTreeMap::from([
            (Value::Text("status".into()), Value::Text("replied".into())),
            (
                Value::Text("reply".into()),
                Value::Map(BTreeMap::from([(
                    Value::Text("arg".into()),
                    Value::Bytes(candid::encode_one(reply).unwrap()),
                )])),
            ),
        ])))
        .unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(10)))
                .unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut length = None;
            loop {
                let mut line = String::new();
                assert!(reader.read_line(&mut line).unwrap() > 0);
                if line == "\r\n" {
                    break;
                }
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    length = Some(value.trim().parse::<usize>().unwrap());
                }
            }
            let mut request = vec![0; length.unwrap()];
            reader.read_exact(&mut request).unwrap();
            let request: Value = serde_cbor::from_slice(&request).unwrap();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
            stream.write_all(&body).unwrap();
            request
        });
        let agent = Agent::builder()
            .with_url(endpoint)
            .with_verify_query_signatures(false)
            .build()
            .unwrap();
        let archive = Icrc3ArchivedBlocks {
            callback: Icrc3ArchiveCallback(Func {
                principal: Principal::anonymous(),
                method: "read_archive_page".into(),
            }),
            args: vec![Icrc3GetBlocksRequest {
                start: Nat::from(12u64),
                length: Nat::from(2u64),
            }],
        };
        let followed = block_on_current_thread(fetch_archive_blocks(&agent, &[archive])).unwrap();
        let Value::Map(envelope) = server.join().unwrap() else {
            panic!("request envelope")
        };
        let Value::Map(content) = &envelope[&Value::Text("content".into())] else {
            panic!("request content")
        };
        assert_eq!(
            content[&Value::Text("request_type".into())],
            Value::Text("query".into())
        );
        assert_eq!(
            content[&Value::Text("method_name".into())],
            Value::Text("read_archive_page".into())
        );
        let Value::Bytes(args) = &content[&Value::Text("arg".into())] else {
            panic!("Candid arguments")
        };
        let args: Vec<Icrc3GetBlocksRequest> = candid::decode_one(args).unwrap();
        assert_eq!(args[0].start, Nat::from(12u64));
        assert_eq!(args[0].length, Nat::from(2u64));
        followed
    }

    #[test]
    fn follows_the_supplied_query_callback_method_and_arguments() {
        let followed = follow_callback_fixture(&[12]);
        assert!(followed.errors.is_empty(), "{:?}", followed.errors);
        assert_eq!(followed.blocks[0].index, "12");
        assert_eq!(followed.blocks[0].callback_method, "read_archive_page");
    }

    #[test]
    fn invalid_archive_rows_are_reported_without_retaining_partial_blocks() {
        for ids in [&[12, 14][..], &[12, 12][..]] {
            let followed = follow_callback_fixture(ids);
            assert_eq!(followed.blocks, Vec::<IcrcFollowedArchiveBlockRow>::new());
            assert_eq!(followed.errors.len(), 1);
            assert_eq!(followed.errors[0].callback_method, "read_archive_page");
            assert_eq!(followed.errors[0].ranges[0].start, "12");
        }
    }

    #[test]
    fn archive_reply_budget_counts_every_reply_and_stops_at_exhaustion() {
        let mut remaining = 10;
        charge_archive_reply(&mut remaining, 4).unwrap();
        assert_eq!(remaining, 6);
        assert!(matches!(
            charge_archive_reply(&mut remaining, 7),
            Err(IcrcError::InvalidTransactionPage { .. })
        ));
        assert_eq!(remaining, 0);
        assert!(charge_archive_reply(&mut remaining, 1).is_err());
        let mut exact = 4;
        charge_archive_reply(&mut exact, 4).unwrap();
        assert_eq!(exact, 0);
    }

    #[test]
    fn exhausted_archive_budget_rejects_work_before_querying() {
        let agent = Agent::builder()
            .with_url("http://127.0.0.1:1")
            .build()
            .unwrap();
        let archive = Icrc3ArchivedBlocks {
            args: vec![Icrc3GetBlocksRequest {
                start: Nat::from(12u32),
                length: Nat::from(1u32),
            }],
            callback: Icrc3ArchiveCallback(Func {
                principal: Principal::anonymous(),
                method: "read_page".into(),
            }),
        };
        let mut remaining = 0;
        let error = block_on_current_thread(query_archive_blocks(&agent, &archive, &mut remaining))
            .unwrap()
            .expect_err("no network work after budget exhaustion");
        assert!(matches!(error, IcrcError::InvalidTransactionPage { .. }));
    }

    #[test]
    fn block_page_validates_request_and_log_bounds_and_disjoint_coverage() {
        let request = IcrcTransactionsRequest {
            source_endpoint: "fixture".into(),
            now_unix_secs: 0,
            ledger_canister_id: Principal::anonymous().to_text(),
            start: 12,
            limit: 2,
            follow_archives: true,
        };
        let range = |start, length| Icrc3GetBlocksRequest {
            start: Nat::from(start),
            length: Nat::from(length),
        };
        let archive = |args| Icrc3ArchivedBlocks {
            args,
            callback: Icrc3ArchiveCallback(Func {
                principal: Principal::anonymous(),
                method: "read_page".into(),
            }),
        };
        let mut result = Icrc3GetBlocksResult {
            log_length: Nat::from(100u32),
            blocks: vec![],
            archived_blocks: vec![archive(vec![range(12u64, 2u64)])],
        };
        validate_block_page(&result, &request).unwrap();
        for length in [0u32, 12, 13] {
            result.log_length = Nat::from(length);
            assert!(matches!(
                validate_block_page(&result, &request),
                Err(IcrcError::InvalidTransactionPage { .. })
            ));
        }
        result.log_length = Nat::from(14u32);
        validate_block_page(&result, &request).unwrap();
        for args in [
            vec![],
            vec![range(12, 0)],
            vec![range(11, 2)],
            vec![range(13, 2)],
            vec![range(12, 1), range(12, 1)],
        ] {
            result.archived_blocks = vec![archive(args)];
            assert!(matches!(
                validate_block_page(&result, &request),
                Err(IcrcError::InvalidTransactionPage { .. })
            ));
        }
        result.archived_blocks = vec![archive(vec![range(12, 2)])];
        result.blocks.push(Icrc3BlockWithId {
            id: Nat::from(12u32),
            block: Icrc3Value::Text("overlap".into()),
        });
        assert!(validate_block_page(&result, &request).is_err());
        result.blocks.clear();
        result.archived_blocks = (0..=MAX_ARCHIVE_CALLBACKS)
            .map(|_| archive(vec![range(12, 1)]))
            .collect();
        assert!(validate_block_page(&result, &request).is_err());
        result.archived_blocks.clear();
        result.blocks = vec![Icrc3BlockWithId {
            id: Nat::from(14u32),
            block: Icrc3Value::Text("outside".into()),
        }];
        assert!(validate_block_page(&result, &request).is_err());
        result.blocks[0].id = Nat::from(13u32);
        result.log_length = Nat::from(13u32);
        assert!(matches!(
            validate_block_page(&result, &request),
            Err(IcrcError::InvalidTransactionPage { .. })
        ));
        result.log_length = Nat::from(14u32);
        validate_block_page(&result, &request).unwrap();
        let huge_request = IcrcTransactionsRequest {
            start: u64::MAX,
            ..request
        };
        result.blocks[0].id = Nat(Nat::from(u64::MAX).0 + 1u32);
        result.log_length = Nat(Nat::from(u64::MAX).0 + 2u32);
        validate_block_page(&result, &huge_request).unwrap();
        result.blocks.clear();
        result.log_length = Nat::from(0u32);
        validate_block_page(&result, &huge_request).expect("empty page past the end of the log");
    }
}
