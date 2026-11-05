//! Module: icrc::live::fetch::capabilities
//!
//! Responsibility: probe bounded ICRC ledger capabilities and retain typed evidence.
//! Does not own: primary report fetches, account history, source traits, caching, or rendering.
//! Boundary: projects shared ledger queries into capability availability rows.

use super::{
    history::{
        ICRC3_GET_ARCHIVES_METHOD, ICRC3_GET_BLOCKS_METHOD, ICRC3_GET_TIP_CERTIFICATE_METHOD,
        ICRC3_SUPPORTED_BLOCK_TYPES_METHOD, query_archives, query_block_types, query_blocks,
        query_tip_certificate,
    },
    live_query_context,
};
use crate::agent_transport::{QueryCallError, QueryFailure};
use crate::candid_call::CandidCallError;
use crate::icrc::{
    ledger::{
        ICRC106_GET_INDEX_PRINCIPAL_METHOD, fetch_icrc_supported_standards,
        index_principal_error_text, query_index_principal,
    },
    model::{
        IcrcCapabilitiesData, IcrcCapabilityRow, IcrcCapabilityStatus, IcrcError, IcrcLedgerRequest,
    },
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::{Nat, Principal};
use ic_platform::icrc_ledger_types::{GetArchivesArgs, GetBlocksRequest};

const ICRC1_SUPPORTED_STANDARDS_METHOD: &str = "icrc1_supported_standards";

pub(in crate::icrc::live) async fn fetch_capabilities_async(
    source: &crate::icrc::LiveIcrcSource,
    request: &IcrcLedgerRequest,
) -> Result<IcrcCapabilitiesData, IcrcError> {
    let (agent, ledger_canister) = live_query_context(
        source,
        &request.source_endpoint,
        &request.ledger_canister_id,
    )?;
    let (standards_result, index, blocks, block_types, archives, tip_certificate) = futures_util::join!(
        fetch_icrc_supported_standards::<CapabilityQueryError>(&agent, &ledger_canister),
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
    capability_result(
        "ICRC-106 index discovery",
        ICRC106_GET_INDEX_PRINCIPAL_METHOD,
        query_index_principal::<CapabilityQueryError>(agent, ledger_canister).await,
        |result| {
            result.map_or_else(index_principal_error_text, |principal| {
                format!("index canister {}", principal.to_text())
            })
        },
    )
}

async fn fetch_blocks_capability(agent: &Agent, ledger_canister: &Principal) -> IcrcCapabilityRow {
    let block_args = vec![GetBlocksRequest {
        start: Nat::from(0_u64),
        length: Nat::from(1_u64),
    }];
    capability_result(
        "ICRC-3 block history",
        ICRC3_GET_BLOCKS_METHOD,
        query_blocks::<CapabilityQueryError>(agent, ledger_canister, &block_args).await,
        |result| {
            format!(
                "log_length {}, returned_blocks {}, archive_callbacks {}",
                result.log_length,
                result.blocks.len(),
                result.archived_blocks.len()
            )
        },
    )
}

async fn fetch_block_types_capability(
    agent: &Agent,
    ledger_canister: &Principal,
) -> IcrcCapabilityRow {
    capability_result(
        "ICRC-3 supported block types",
        ICRC3_SUPPORTED_BLOCK_TYPES_METHOD,
        query_block_types::<CapabilityQueryError>(agent, ledger_canister).await,
        |block_types| {
            named_count_detail(
                "block type",
                block_types
                    .iter()
                    .map(|block_type| block_type.block_type.as_str()),
            )
        },
    )
}

async fn fetch_archives_capability(
    agent: &Agent,
    ledger_canister: &Principal,
) -> IcrcCapabilityRow {
    let args = GetArchivesArgs { from: None };
    capability_result(
        "ICRC-3 archive discovery",
        ICRC3_GET_ARCHIVES_METHOD,
        query_archives::<CapabilityQueryError>(agent, ledger_canister, &args).await,
        |archives| format!("{} archive range(s)", archives.len()),
    )
}

async fn fetch_tip_certificate_capability(
    agent: &Agent,
    ledger_canister: &Principal,
) -> IcrcCapabilityRow {
    capability_result(
        "ICRC-3 tip certificate",
        ICRC3_GET_TIP_CERTIFICATE_METHOD,
        query_tip_certificate::<CapabilityQueryError>(agent, ledger_canister).await,
        |result| match result {
            Some(certificate) => {
                format!(
                    "verified certificate {} bytes, hash tree {} bytes",
                    certificate.certificate.len(),
                    certificate.hash_tree.len()
                )
            }
            None => "certificate absent".to_string(),
        },
    )
}

fn capability_result<T>(
    capability: &str,
    method: &'static str,
    result: Result<T, CapabilityQueryError>,
    describe: impl FnOnce(T) -> String,
) -> IcrcCapabilityRow {
    match result {
        Ok(value) => available_capability_row(capability, method, describe(value)),
        Err(error) => error_capability_row(capability, method, error),
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
        status: IcrcCapabilityStatus::Available,
        details: Some(details.into()),
        error: None,
    }
}

///
/// CapabilityQueryError
///
/// Private query failure retaining a protocol-derived classification and original diagnostic.
///

struct CapabilityQueryError {
    error: IcrcError,
    method_not_exported: bool,
}

impl From<IcrcError> for CapabilityQueryError {
    fn from(error: IcrcError) -> Self {
        Self {
            error,
            method_not_exported: false,
        }
    }
}

impl CandidCallError for CapabilityQueryError {
    fn candid_encode(message: &'static str, reason: String) -> Self {
        IcrcError::CandidEncode { message, reason }.into()
    }

    fn agent_call(method: &'static str, error: QueryCallError) -> Self {
        // IC reject code 5 is CanisterError; free-form text is not a code.
        let method_not_exported = matches!(
            error.failure(),
            QueryFailure::Rejected {
                code: 5,
                error_code: Some("IC0536"),
                ..
            }
        );
        Self {
            error: IcrcError::AgentCall {
                method,
                reason: error.to_string(),
            },
            method_not_exported,
        }
    }

    fn candid_decode(message: &'static str, reason: String) -> Self {
        IcrcError::CandidDecode { message, reason }.into()
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Consumes the owned query error at the capability-to-report boundary."
)]
fn error_capability_row(
    capability: &str,
    method: &'static str,
    error: CapabilityQueryError,
) -> IcrcCapabilityRow {
    let (status, details) = if error.method_not_exported {
        (
            IcrcCapabilityStatus::Unsupported,
            "method not exported by target canister",
        )
    } else {
        (IcrcCapabilityStatus::Error, "query failed")
    };
    IcrcCapabilityRow {
        capability: capability.to_string(),
        method: method.to_string(),
        status,
        details: Some(details.to_string()),
        error: Some(error.error.to_string()),
    }
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

#[cfg(test)]
mod tests;
