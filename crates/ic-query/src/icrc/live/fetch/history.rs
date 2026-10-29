//! Module: icrc::live::fetch::history
//!
//! Responsibility: query and project ICRC index, block, archive, and tip evidence.
//! Does not own: account point values, capability policy, source traits, caching, or rendering.
//! Boundary: owns ICRC-3 history traversal and wire-to-report conversion.

use super::{super::tip_certificate::verified_tip_certificate_data, live_query_context};
use crate::candid_call::{CandidCallError, query_canister, query_no_args};
use crate::decimal::is_unsigned_decimal;
use crate::{
    hex::hex_bytes,
    icrc::{
        ledger::{
            index_principal_error_text, nat_text, principal_from_text, query_index_principal,
        },
        model::{
            IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow,
            IcrcArchivesRequest, IcrcBlockTypeRow, IcrcError, IcrcFollowedArchiveBlockRow,
            IcrcIndexData, IcrcLedgerRequest, IcrcTransactionBlockRow, IcrcTransactionsData,
            IcrcTransactionsRequest,
        },
    },
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::{Nat, Principal};
use ic_platform::icrc_ledger_types::{
    ArchivedBlocks, BlockWithId, GetArchivesArgs, GetBlocksRequest, GetBlocksResult,
    ICRC3ArchiveInfo, ICRC3DataCertificate, ICRC3Value, SupportedBlockType,
};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};

const MAX_ARCHIVE_CALLBACKS: usize = 100;
const MAX_ARCHIVE_REPLY_BYTES: usize = 64 * 1024 * 1024;

pub(super) const ICRC3_GET_BLOCKS_METHOD: &str = "icrc3_get_blocks";
pub(super) const ICRC3_SUPPORTED_BLOCK_TYPES_METHOD: &str = "icrc3_supported_block_types";
pub(super) const ICRC3_GET_ARCHIVES_METHOD: &str = "icrc3_get_archives";
pub(super) const ICRC3_GET_TIP_CERTIFICATE_METHOD: &str = "icrc3_get_tip_certificate";

pub(in crate::icrc::live) async fn fetch_index_async(
    source: &crate::icrc::LiveIcrcSource,
    request: &IcrcLedgerRequest,
) -> Result<IcrcIndexData, IcrcError> {
    let (agent, ledger_canister) = live_query_context(
        source,
        &request.source_endpoint,
        &request.ledger_canister_id,
    )?;
    let result = query_index_principal::<IcrcError>(&agent, &ledger_canister).await?;

    Ok(match result {
        Ok(principal) => IcrcIndexData {
            index_canister_id: Some(principal.to_text()),
            index_error: None,
        },
        Err(error) => IcrcIndexData {
            index_canister_id: None,
            index_error: Some(index_principal_error_text(error)),
        },
    })
}

pub(in crate::icrc::live) async fn fetch_transactions_async(
    source: &crate::icrc::LiveIcrcSource,
    request: &IcrcTransactionsRequest,
) -> Result<IcrcTransactionsData, IcrcError> {
    let (agent, ledger_canister) = live_query_context(
        source,
        &request.ledger.source_endpoint,
        &request.ledger.ledger_canister_id,
    )?;
    let block_args = vec![GetBlocksRequest {
        start: Nat::from(request.start),
        length: Nat::from(request.limit),
    }];
    let result = query_blocks::<IcrcError>(&agent, &ledger_canister, &block_args).await?;
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

async fn fetch_archive_blocks(agent: &Agent, archives: &[ArchivedBlocks]) -> ArchiveFollowResult {
    let mut result = ArchiveFollowResult::default();
    let mut remaining_bytes = MAX_ARCHIVE_REPLY_BYTES;
    for archive in archives {
        let canister_id = archive.callback.canister_id.to_text();
        let method = archive.callback.method.clone();
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
    archive: &ArchivedBlocks,
    remaining_bytes: &mut usize,
) -> Result<GetBlocksResult, IcrcError> {
    const CONTEXT: &str = "ICRC3 archive callback";
    if *remaining_bytes == 0 {
        return Err(invalid_transaction_page(
            "archive reply byte budget exhausted",
        ));
    }
    let arg = ic_platform::candid::contract::encode_one(&archive.args).map_err(|error| {
        IcrcError::CandidEncode {
            message: CONTEXT,
            reason: error.to_string(),
        }
    })?;
    let bytes = crate::agent_transport::query(
        agent,
        &archive.callback.canister_id,
        &archive.callback.method,
        arg,
    )
    .await
    .map_err(|error| IcrcError::AgentCall {
        method: CONTEXT,
        reason: error.to_string(),
    })?;
    charge_archive_reply(remaining_bytes, bytes.len())?;
    let result: GetBlocksResult =
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
    result: &GetBlocksResult,
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
        if !is_unsigned_decimal(text) {
            return Err(invalid_transaction_page(
                "log length, block indexes, and ranges must be decimal naturals",
            ));
        }
        text.parse()
            .map_err(|_| invalid_transaction_page("invalid decimal block index or range"))
    };
    let log_length = data.log_length.as_deref().map(parse).transpose()?;
    let mut ranges = Vec::new();
    let mut callback_ranges = BTreeMap::<_, Vec<_>>::new();
    for block in &data.blocks {
        let id = parse(&block.index)?;
        let end = Nat(&id.0 + 1u32);
        ranges.push((id, end));
    }
    for archive in &data.archived_blocks {
        if archive.ranges.is_empty() {
            return Err(invalid_transaction_page("archive callback has no ranges"));
        }
        for range in &archive.ranges {
            let start = parse(&range.start)?;
            let length = parse(&range.length)?;
            let end = Nat(&start.0 + length.0);
            ranges.push((start.clone(), end.clone()));
            callback_ranges
                .entry((
                    archive.callback_canister_id.as_str(),
                    archive.callback_method.as_str(),
                ))
                .or_default()
                .push(start..end);
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
    // Disjoint validated ranges already exclude resident ids from archive rows.
    let mut followed_ids = BTreeSet::new();
    for block in &data.followed_archive_blocks {
        let id = parse(&block.index)?;
        let in_range = callback_ranges
            .get(&(
                block.archive_canister_id.as_str(),
                block.callback_method.as_str(),
            ))
            .is_some_and(|ranges| ranges.iter().any(|range| range.contains(&id)));
        if !in_range || !followed_ids.insert(id) {
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
    result: &GetBlocksResult,
    args: &[GetBlocksRequest],
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
    source: &crate::icrc::LiveIcrcSource,
    request: &IcrcLedgerRequest,
) -> Result<Vec<IcrcBlockTypeRow>, IcrcError> {
    let (agent, ledger_canister) = live_query_context(
        source,
        &request.source_endpoint,
        &request.ledger_canister_id,
    )?;
    let block_types = query_block_types::<IcrcError>(&agent, &ledger_canister).await?;

    Ok(block_types
        .into_iter()
        .map(block_type_row_from_wire)
        .collect())
}

pub(in crate::icrc::live) async fn fetch_archives_async(
    source: &crate::icrc::LiveIcrcSource,
    request: &IcrcArchivesRequest,
) -> Result<Vec<IcrcArchiveRow>, IcrcError> {
    let (agent, ledger_canister) = live_query_context(
        source,
        &request.ledger.source_endpoint,
        &request.ledger.ledger_canister_id,
    )?;
    let args = GetArchivesArgs {
        from: request
            .from_canister_id
            .as_deref()
            .map(|from| principal_from_text(from, "from_canister_id"))
            .transpose()?,
    };
    let archives = query_archives::<IcrcError>(&agent, &ledger_canister, &args).await?;

    Ok(archives.into_iter().map(archive_row_from_wire).collect())
}

pub(in crate::icrc::live) async fn fetch_tip_certificate_async(
    source: &crate::icrc::LiveIcrcSource,
    request: &IcrcLedgerRequest,
) -> Result<Option<ic_platform::icrc_ledger_types::ICRC3DataCertificate>, IcrcError> {
    let (agent, ledger_canister) = live_query_context(
        source,
        &request.source_endpoint,
        &request.ledger_canister_id,
    )?;
    query_tip_certificate::<IcrcError>(&agent, &ledger_canister).await
}

pub(super) async fn query_blocks<E>(
    agent: &Agent,
    canister: &Principal,
    args: &[GetBlocksRequest],
) -> Result<GetBlocksResult, E>
where
    E: CandidCallError,
{
    query_canister::<_, GetBlocksResult, E>(
        agent,
        canister,
        ICRC3_GET_BLOCKS_METHOD,
        ICRC3_GET_BLOCKS_METHOD,
        ICRC3_GET_BLOCKS_METHOD,
        &args,
    )
    .await
}

pub(super) async fn query_block_types<E: CandidCallError>(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<Vec<SupportedBlockType>, E> {
    query_no_args::<Vec<SupportedBlockType>, E>(
        agent,
        ledger_canister,
        ICRC3_SUPPORTED_BLOCK_TYPES_METHOD,
    )
    .await
}

pub(super) async fn query_archives<E: CandidCallError>(
    agent: &Agent,
    ledger_canister: &Principal,
    args: &GetArchivesArgs,
) -> Result<Vec<ICRC3ArchiveInfo>, E> {
    query_canister::<GetArchivesArgs, Vec<ICRC3ArchiveInfo>, E>(
        agent,
        ledger_canister,
        ICRC3_GET_ARCHIVES_METHOD,
        ICRC3_GET_ARCHIVES_METHOD,
        ICRC3_GET_ARCHIVES_METHOD,
        args,
    )
    .await
}

pub(super) async fn query_tip_certificate<E: CandidCallError + From<IcrcError>>(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<Option<ICRC3DataCertificate>, E> {
    let certificate = query_no_args::<Option<ICRC3DataCertificate>, E>(
        agent,
        ledger_canister,
        ICRC3_GET_TIP_CERTIFICATE_METHOD,
    )
    .await?;
    verified_tip_certificate_data(agent, ledger_canister, certificate).map_err(E::from)
}

fn transactions_data_from_blocks(
    result: GetBlocksResult,
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

fn transaction_block_row_from_wire(block: BlockWithId) -> IcrcTransactionBlockRow {
    let block_type = icrc3_text_at_path(&block.block, &["btype"]);
    IcrcTransactionBlockRow {
        index: nat_text(&block.id),
        transaction_kind: block_type
            .clone()
            .or_else(|| icrc3_text_at_path(&block.block, &["tx", "op"])),
        block_type,
        timestamp_unix_nanos: icrc3_nat_at_path(&block.block, &["ts"]),
        amount_base_units: icrc3_nat_at_path(&block.block, &["tx", "amt"]),
        raw_block: icrc3_value_json(block.block),
    }
}

fn archived_blocks_row_from_wire(archive: ArchivedBlocks) -> IcrcArchivedBlocksRow {
    let ArchivedBlocks { args, callback } = archive;
    IcrcArchivedBlocksRow {
        callback_canister_id: callback.canister_id.to_text(),
        callback_method: callback.method,
        ranges: archived_range_rows(&args),
    }
}

fn followed_archive_block_row_from_wire(
    archive_canister_id: &str,
    callback_method: &str,
    block: BlockWithId,
) -> IcrcFollowedArchiveBlockRow {
    let summary = transaction_block_row_from_wire(block);
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

fn archive_follow_error_row(archive: &ArchivedBlocks, error: String) -> IcrcArchiveFollowErrorRow {
    IcrcArchiveFollowErrorRow {
        callback_canister_id: archive.callback.canister_id.to_text(),
        callback_method: archive.callback.method.clone(),
        ranges: archived_range_rows(&archive.args),
        error,
    }
}

fn archived_range_rows(ranges: &[GetBlocksRequest]) -> Vec<IcrcArchivedRangeRow> {
    ranges
        .iter()
        .map(|range| IcrcArchivedRangeRow {
            start: nat_text(&range.start),
            length: nat_text(&range.length),
        })
        .collect()
}

fn block_type_row_from_wire(block_type: SupportedBlockType) -> IcrcBlockTypeRow {
    IcrcBlockTypeRow {
        block_type: block_type.block_type,
        url: block_type.url,
    }
}

fn archive_row_from_wire(archive: ICRC3ArchiveInfo) -> IcrcArchiveRow {
    IcrcArchiveRow {
        canister_id: archive.canister_id.to_text(),
        start: nat_text(&archive.start),
        end: nat_text(&archive.end),
    }
}

fn icrc3_text_at_path(value: &ICRC3Value, path: &[&str]) -> Option<String> {
    let value = icrc3_value_at_path(value, path)?;
    match value {
        ICRC3Value::Text(text) => Some(text.clone()),
        _ => None,
    }
}

fn icrc3_nat_at_path(value: &ICRC3Value, path: &[&str]) -> Option<String> {
    let value = icrc3_value_at_path(value, path)?;
    match value {
        ICRC3Value::Nat(nat) => Some(nat_text(nat)),
        _ => None,
    }
}

fn icrc3_value_at_path<'a>(value: &'a ICRC3Value, path: &[&str]) -> Option<&'a ICRC3Value> {
    path.iter().try_fold(value, |value, key| match value {
        ICRC3Value::Map(map) => map.get(*key),
        _ => None,
    })
}

fn icrc3_value_json(value: ICRC3Value) -> JsonValue {
    let (kind, value) = match value {
        ICRC3Value::Blob(bytes) => ("Blob", JsonValue::String(hex_bytes(&bytes))),
        ICRC3Value::Text(text) => ("Text", JsonValue::String(text)),
        ICRC3Value::Nat(nat) => ("Nat", JsonValue::String(nat_text(&nat))),
        ICRC3Value::Int(int) => ("Int", JsonValue::String(int.0.to_str_radix(10))),
        ICRC3Value::Array(values) => (
            "Array",
            JsonValue::Array(values.into_iter().map(icrc3_value_json).collect()),
        ),
        ICRC3Value::Map(entries) => (
            "Map",
            JsonValue::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, icrc3_value_json(value)))
                    .collect(),
            ),
        ),
    };
    JsonValue::Object(JsonMap::from_iter([(kind.to_string(), value)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
    use ic_platform::icrc_ledger_types::QueryArchiveFn;
    use serde_cbor::Value;
    use std::{collections::BTreeMap, time::Duration};

    #[test]
    fn wire_history_projects_plain_decimal_numbers_accepted_by_report_validation() {
        let large: Nat = "18446744073709551616".parse().unwrap();
        let block = BlockWithId {
            id: Nat::from(1_000u32),
            block: ICRC3Value::Map(BTreeMap::from([
                ("ts".into(), ICRC3Value::Nat(large.clone())),
                (
                    "tx".into(),
                    ICRC3Value::Map(BTreeMap::from([(
                        "amt".into(),
                        ICRC3Value::Nat(large.clone()),
                    )])),
                ),
                (
                    "signed".into(),
                    ICRC3Value::Int(ic_platform::candid::contract::Int::from(-1_000)),
                ),
                ("blob".into(), ICRC3Value::Blob(vec![0, 0xab, 0xff].into())),
                (
                    "array".into(),
                    ICRC3Value::Array(vec![ICRC3Value::Text("fixture".into())]),
                ),
            ])),
        };
        let result = GetBlocksResult {
            log_length: large,
            blocks: vec![block.clone()],
            archived_blocks: vec![ArchivedBlocks {
                args: vec![GetBlocksRequest {
                    start: Nat::from(1_001u32),
                    length: Nat::from(1_000u32),
                }],
                callback: QueryArchiveFn::new(Principal::anonymous(), "read_page"),
            }],
        };
        let request = IcrcTransactionsRequest {
            ledger: crate::icrc::IcrcLedgerRequest::new(
                "fixture",
                0,
                Principal::anonymous().to_text(),
            ),
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
        assert_eq!(data.blocks[0].raw_block["Map"]["blob"]["Blob"], "00abff");
        assert_eq!(
            data.blocks[0].raw_block["Map"]["array"]["Array"][0]["Text"],
            "fixture"
        );
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
        let archive = archive_row_from_wire(ICRC3ArchiveInfo {
            canister_id: Principal::anonymous(),
            start: Nat::from(1_000u32),
            end: Nat::from(2_000u32),
        });
        assert_eq!(archive.start, "1000");
        assert_eq!(archive.end, "2000");
    }

    #[test]
    fn followed_blocks_remain_bound_to_each_callbacks_disjoint_ranges() {
        let canister = Principal::anonymous();
        let archive = |start: u32, method: &str| ArchivedBlocks {
            args: vec![GetBlocksRequest {
                start: Nat::from(start),
                length: Nat::from(1u32),
            }],
            callback: QueryArchiveFn::new(canister, method),
        };
        let followed = |index: u32, method: &str| {
            followed_archive_block_row_from_wire(
                &canister.to_text(),
                method,
                BlockWithId {
                    id: Nat::from(index),
                    block: ICRC3Value::Nat(Nat::from(index)),
                },
            )
        };
        let request = IcrcTransactionsRequest {
            ledger: crate::icrc::IcrcLedgerRequest::new("fixture", 0, canister.to_text()),
            start: 0,
            limit: 4,
            follow_archives: true,
        };
        let data = transactions_data_from_blocks(
            GetBlocksResult {
                log_length: Nat::from(4u32),
                blocks: vec![],
                archived_blocks: vec![
                    archive(0, "first"),
                    archive(2, "first"),
                    archive(1, "second"),
                    archive(3, "second"),
                ],
            },
            ArchiveFollowResult {
                blocks: vec![
                    followed(0, "first"),
                    followed(2, "first"),
                    followed(1, "second"),
                    followed(3, "second"),
                ],
                errors: vec![],
            },
        );
        validate_transactions_data(&request, &data)
            .expect("all ranges for the same callback are retained");
        let mut wrong_canister = followed(0, "first");
        wrong_canister.archive_canister_id = Principal::management_canister().to_text();
        for rejected in [followed(0, "second"), followed(4, "second"), wrong_canister] {
            let mut invalid = data.clone();
            invalid.followed_archive_blocks = vec![rejected];
            assert!(matches!(
                validate_transactions_data(&request, &invalid),
                Err(IcrcError::InvalidTransactionPage { .. })
            ));
        }
        let mut duplicated = data;
        duplicated
            .followed_archive_blocks
            .push(followed(0, "first"));
        assert!(matches!(
            validate_transactions_data(&request, &duplicated),
            Err(IcrcError::InvalidTransactionPage { .. })
        ));
    }

    async fn follow_callback_fixture(block_ids: &[u64]) -> ArchiveFollowResult {
        let reply = GetBlocksResult {
            log_length: Nat::from(100u64),
            blocks: block_ids
                .iter()
                .map(|id| BlockWithId {
                    id: Nat::from(*id),
                    block: ICRC3Value::Text("fixture".into()),
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
                    Value::Bytes(ic_platform::candid::contract::encode_one(reply).unwrap()),
                )])),
            ),
        ])))
        .unwrap();
        let server = HttpFixture::spawn(
            &body,
            "application/cbor",
            HttpFixtureLimits {
                timeout: Duration::from_secs(10),
                max_header_bytes: 64 * 1024,
                max_body_bytes: 8 * 1024 * 1024,
            },
        )
        .unwrap();
        let endpoint = server.endpoint();
        let agent = Agent::builder()
            .with_url(endpoint)
            .with_verify_query_signatures(false)
            .build()
            .unwrap();
        let archive = ArchivedBlocks {
            callback: QueryArchiveFn::new(Principal::anonymous(), "read_archive_page"),
            args: vec![GetBlocksRequest {
                start: Nat::from(12u64),
                length: Nat::from(2u64),
            }],
        };
        let followed = fetch_archive_blocks(&agent, &[archive]).await;
        let raw = server.finish().unwrap();
        let body_start = raw
            .windows(4)
            .position(|bytes| bytes == b"\r\n\r\n")
            .unwrap()
            + 4;
        let Value::Map(envelope) = serde_cbor::from_slice::<Value>(&raw[body_start..]).unwrap()
        else {
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
        let args: Vec<GetBlocksRequest> = crate::candid_decode::decode_reply(args).unwrap();
        assert_eq!(args[0].start, Nat::from(12u64));
        assert_eq!(args[0].length, Nat::from(2u64));
        followed
    }

    #[tokio::test(flavor = "current_thread")]
    async fn follows_the_supplied_query_callback_method_and_arguments() {
        let followed = follow_callback_fixture(&[12]).await;
        assert!(followed.errors.is_empty(), "{:?}", followed.errors);
        assert_eq!(followed.blocks[0].index, "12");
        assert_eq!(followed.blocks[0].callback_method, "read_archive_page");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn invalid_archive_rows_are_reported_without_retaining_partial_blocks() {
        for ids in [&[12, 14][..], &[12, 12][..]] {
            let followed = follow_callback_fixture(ids).await;
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

    #[tokio::test(flavor = "current_thread")]
    async fn exhausted_archive_budget_rejects_work_before_querying() {
        let agent = Agent::builder()
            .with_url("http://127.0.0.1:1")
            .build()
            .unwrap();
        let archive = ArchivedBlocks {
            args: vec![GetBlocksRequest {
                start: Nat::from(12u32),
                length: Nat::from(1u32),
            }],
            callback: QueryArchiveFn::new(Principal::anonymous(), "read_page"),
        };
        let mut remaining = 0;
        let error = query_archive_blocks(&agent, &archive, &mut remaining)
            .await
            .expect_err("no network work after budget exhaustion");
        assert!(matches!(error, IcrcError::InvalidTransactionPage { .. }));
    }

    #[test]
    fn block_page_validates_request_and_log_bounds_and_disjoint_coverage() {
        let request = IcrcTransactionsRequest {
            ledger: crate::icrc::IcrcLedgerRequest::new(
                "fixture",
                0,
                Principal::anonymous().to_text(),
            ),
            start: 12,
            limit: 2,
            follow_archives: true,
        };
        let range = |start, length| GetBlocksRequest {
            start: Nat::from(start),
            length: Nat::from(length),
        };
        let archive = |args| ArchivedBlocks {
            args,
            callback: QueryArchiveFn::new(Principal::anonymous(), "read_page"),
        };
        let mut result = GetBlocksResult {
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
        result.blocks.push(BlockWithId {
            id: Nat::from(12u32),
            block: ICRC3Value::Text("overlap".into()),
        });
        assert!(validate_block_page(&result, &request).is_err());
        result.blocks.clear();
        result.archived_blocks = (0..=MAX_ARCHIVE_CALLBACKS)
            .map(|_| archive(vec![range(12, 1)]))
            .collect();
        assert!(validate_block_page(&result, &request).is_err());
        result.archived_blocks.clear();
        result.blocks = vec![BlockWithId {
            id: Nat::from(14u32),
            block: ICRC3Value::Text("outside".into()),
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
