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
use crate::candid_call::CandidCallError;
use crate::icrc::{
    ledger::{GetIndexPrincipalResult, fetch_icrc_supported_standards, index_principal_error_text},
    model::{
        IcrcCapabilitiesData, IcrcCapabilityRow, IcrcCapabilityStatus, IcrcError,
        IcrcLedgerRequest, IcrcTipCertificateData,
    },
};
use candid::{Nat, Principal};
use ic_agent::{Agent, AgentError, agent::RejectCode};
use icrc_ledger_types::icrc3::archive::GetArchivesArgs;
use icrc_ledger_types::icrc3::blocks::GetBlocksRequest;

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
    let (standards_result, index, blocks, block_types, archives, tip_certificate) = futures::join!(
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
    match query_index_principal::<CapabilityQueryError>(agent, ledger_canister).await {
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
    let block_args = vec![GetBlocksRequest {
        start: Nat::from(0_u64),
        length: Nat::from(1_u64),
    }];
    match query_blocks::<_, CapabilityQueryError>(agent, ledger_canister, &block_args).await {
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
    match query_block_types::<CapabilityQueryError>(agent, ledger_canister).await {
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
    let args = GetArchivesArgs { from: None };
    match query_archives::<CapabilityQueryError>(agent, ledger_canister, &args).await {
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
    match query_tip_certificate::<CapabilityQueryError>(agent, ledger_canister).await {
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

    fn agent_call(method: &'static str, error: AgentError) -> Self {
        let method_not_exported = match &error {
            AgentError::CertifiedReject { reject, .. }
            | AgentError::UncertifiedReject { reject, .. } => {
                // IC0536 is CanisterMethodNotFound; free-form reject text is not a code.
                reject.reject_code == RejectCode::CanisterError
                    && reject.error_code.as_deref() == Some("IC0536")
            }
            _ => false,
        };
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
mod tests {
    use super::*;
    use ic_testkit::testing::read_http_request;
    use serde_cbor::Value;
    use std::{collections::BTreeMap, io::Write, net::TcpListener, thread, time::Duration};

    #[tokio::test(flavor = "current_thread")]
    async fn capability_probe_retains_reject_metadata_through_the_ledger_transport() {
        for (error_code, message, expected) in [
            (
                "IC0536",
                "fixture missing method",
                IcrcCapabilityStatus::Unsupported,
            ),
            (
                "IC0502",
                "IC0536 method not found",
                IcrcCapabilityStatus::Error,
            ),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}", listener.local_addr().unwrap());
            let body = serde_cbor::to_vec(&Value::Map(BTreeMap::from([
                (Value::Text("status".into()), Value::Text("rejected".into())),
                (Value::Text("reject_code".into()), Value::Integer(5)),
                (
                    Value::Text("reject_message".into()),
                    Value::Text(message.into()),
                ),
                (
                    Value::Text("error_code".into()),
                    Value::Text(error_code.into()),
                ),
            ])))
            .unwrap();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_http_request(
                    &mut stream,
                    Duration::from_secs(10),
                    64 * 1024,
                    8 * 1024 * 1024,
                )
                .unwrap();
                let body_start = request
                    .windows(4)
                    .position(|bytes| bytes == b"\r\n\r\n")
                    .unwrap()
                    + 4;
                let request = request[body_start..].to_vec();
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                stream.write_all(&body).unwrap();
                request
            });
            let agent = Agent::builder()
                .with_url(endpoint)
                .with_verify_query_signatures(false)
                .build()
                .unwrap();
            let row = fetch_index_capability(&agent, &Principal::anonymous()).await;
            let request: Value = serde_cbor::from_slice(&server.join().unwrap()).unwrap();
            let Value::Map(envelope) = request else {
                panic!("request envelope")
            };
            let Value::Map(content) = &envelope[&Value::Text("content".into())] else {
                panic!("request content")
            };
            assert_eq!(
                content[&Value::Text("method_name".into())],
                Value::Text(ICRC106_GET_INDEX_PRINCIPAL_METHOD.into())
            );
            assert_eq!(row.status, expected);
            let diagnostic = row.error.unwrap();
            assert!(diagnostic.contains(message));
            assert!(diagnostic.contains(error_code));
        }
    }

    #[test]
    fn capability_failures_classify_protocol_codes_and_preserve_diagnostics() {
        for certified in [false, true] {
            for (code, error_code, message, expected) in [
                (
                    RejectCode::CanisterError,
                    Some("IC0536"),
                    "method unavailable",
                    IcrcCapabilityStatus::Unsupported,
                ),
                (
                    RejectCode::CanisterError,
                    Some("IC0502"),
                    "IC0536: method not found",
                    IcrcCapabilityStatus::Error,
                ),
                (
                    RejectCode::CanisterError,
                    None,
                    "Canister has no query method",
                    IcrcCapabilityStatus::Error,
                ),
                (
                    RejectCode::CanisterReject,
                    Some("IC0536"),
                    "method not found",
                    IcrcCapabilityStatus::Error,
                ),
            ] {
                let reject = ic_agent::agent::RejectResponse {
                    reject_code: code,
                    reject_message: message.to_string(),
                    error_code: error_code.map(str::to_string),
                };
                let error = if certified {
                    AgentError::CertifiedReject {
                        reject,
                        operation: None,
                    }
                } else {
                    AgentError::UncertifiedReject {
                        reject,
                        operation: None,
                    }
                };
                let original = IcrcError::AgentCall {
                    method: ICRC3_GET_BLOCKS_METHOD,
                    reason: error.to_string(),
                }
                .to_string();
                let row = error_capability_row(
                    "ICRC-3 block history",
                    ICRC3_GET_BLOCKS_METHOD,
                    CapabilityQueryError::agent_call(ICRC3_GET_BLOCKS_METHOD, error),
                );
                assert_eq!(row.status, expected);
                assert_eq!(row.error.as_deref(), Some(original.as_str()));
            }
        }
        let error = CapabilityQueryError::candid_decode(
            ICRC3_GET_BLOCKS_METHOD,
            "method not found IC0536".into(),
        );
        assert_eq!(
            error_capability_row("blocks", ICRC3_GET_BLOCKS_METHOD, error).status,
            IcrcCapabilityStatus::Error
        );
    }
}
