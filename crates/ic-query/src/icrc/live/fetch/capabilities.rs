//! Module: icrc::live::fetch::capabilities
//!
//! Responsibility: probe bounded ICRC ledger capabilities and retain typed evidence.
//! Does not own: primary report fetches, account history, source traits, caching, or rendering.
//! Boundary: projects shared ledger queries into capability availability rows.

use super::{
    account::token_standard_row_from_ledger,
    history::{
        ICRC3_GET_ARCHIVES_METHOD, ICRC3_GET_BLOCKS_METHOD, ICRC3_GET_TIP_CERTIFICATE_METHOD,
        ICRC3_SUPPORTED_BLOCK_TYPES_METHOD, ICRC106_GET_INDEX_PRINCIPAL_METHOD, query_archives,
        query_block_types, query_blocks, query_index_principal, query_tip_certificate,
    },
    live_query_context,
};
use crate::icrc::{
    ledger::{
        GetIndexPrincipalResult, Icrc3GetArchivesArgs, Icrc3GetBlocksRequest,
        fetch_icrc_supported_standards, index_principal_error_text,
    },
    model::{
        IcrcCapabilitiesData, IcrcCapabilityRow, IcrcError, IcrcLedgerRequest,
        IcrcTipCertificateData,
    },
};
use candid::{Nat, Principal};
use ic_agent::Agent;

const ICRC1_SUPPORTED_STANDARDS_METHOD: &str = "icrc1_supported_standards";

pub(in crate::icrc::live) async fn fetch_capabilities_async(
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
