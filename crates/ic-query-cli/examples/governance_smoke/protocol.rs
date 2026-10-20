//! Direct IC protocol calls and bounded admission of original reply evidence.

use crate::{Result, command, invalid};
use ic_host_artifacts::{
    artifact::Sha256Digest,
    wasm::{self, InspectionLimits},
};
use ic_host_fs::{
    durable::{PublicationMode, WriteOptions, write_bytes},
    read::read_file,
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;
use std::{future::Future, io, path::Path, time::Duration};
use url::Url;

pub const WASM_BYTES: usize = 64 * 1024 * 1024;
const REPLY_BYTES: usize = 2 * 1024 * 1024;

/// Read bounded regular-file bytes and inspect their Wasm structure once.
pub fn read_wasm(path: &Path) -> Result<Vec<u8>> {
    let bytes = read_file(path, WASM_BYTES)?;
    wasm::inspect(
        &bytes,
        InspectionLimits {
            module_bytes: WASM_BYTES,
            sections: 10_000,
            exports: 10_000,
            custom_sections: 1_000,
        },
    )?;
    Ok(bytes)
}

pub const REPORT_METHOD: u64 = 1;
pub const REQUEST_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 128,
    allocation: 2048,
    nodes: 4,
    depth: 4,
};
pub const RESPONSE_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: REPLY_BYTES,
    allocation: 24 * 1024 * 1024,
    nodes: 4,
    depth: 4,
};

pub fn encode_request(kind: &str) -> Result<Vec<u8>> {
    Ok(ic_wire::native::encode_request(
        REPORT_METHOD,
        kind,
        REQUEST_LIMITS,
    )?)
}

pub fn decode_reply(bytes: &[u8]) -> Result<String> {
    ic_wire::native::decode_response(REPORT_METHOD, bytes, RESPONSE_LIMITS)
        .map_err(|error| invalid(&format!("invalid probe Wire reply: {error}")))
}

pub fn publish_reply(path: &Path, bytes: &[u8]) -> Result<String> {
    // Evidence is private and create-new, including replies rejected by admission.
    write_bytes(
        path,
        WriteOptions {
            mode: PublicationMode::CreateNew,
            permissions: 0o600,
        },
        bytes,
    )?;
    decode_reply(bytes)
}

pub fn agent(environment: &str, endpoint: &str) -> Result<Agent> {
    let endpoint = Url::parse(endpoint)?;
    let clean = endpoint.username().is_empty()
        && endpoint.password().is_none()
        && endpoint.query().is_none()
        && endpoint.fragment().is_none();
    let allowed = match environment {
        "local" => {
            endpoint.scheme() == "http"
                && matches!(
                    endpoint.host_str(),
                    Some("localhost" | "127.0.0.1" | "[::1]")
                )
        }
        "mainnet-smoke" => endpoint.scheme() == "https" && endpoint.host_str().is_some(),
        _ => false,
    };
    if !clean || !allowed {
        return Err(invalid(
            "endpoint does not match the selected network trust policy",
        ));
    }
    Ok(Agent::builder()
        .with_url(endpoint.as_str())
        .with_http_client(
            reqwest::Client::builder()
                .use_rustls_tls()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(360))
                .build()?,
        )
        .with_max_response_body_size(8 * 1024 * 1024)
        .build()?)
}

///
/// Probe
///
/// Certified metadata and raw report replies from an explicitly selected principal.
///

pub trait Probe {
    fn module_hash(&mut self) -> Result<String>;
    fn metadata(&mut self, name: &str) -> Result<Vec<u8>>;
    fn report(&mut self, kind: &str) -> Result<Vec<u8>>;
}

///
/// AgentProbe
///
/// Development-only agent with explicit network trust and bounded call deadlines.
///

pub struct AgentProbe {
    agent: Agent,
    principal: Principal,
    local: bool,
    runtime: tokio::runtime::Runtime,
}

impl AgentProbe {
    pub fn new(environment: &str, endpoint: &str, principal: &str) -> Result<Self> {
        Ok(Self {
            agent: agent(environment, endpoint)?,
            principal: Principal::from_text(principal)?,
            local: environment == "local",
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?,
        })
    }

    fn call<T>(&self, operation: impl Future<Output = Result<T>>) -> Result<T> {
        self.runtime.block_on(async {
            let cancelled = async {
                loop {
                    if let Some(signal) = command::interruption() {
                        return invalid(&format!(
                            "interrupted by {}",
                            command::signal_name(signal)
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            };
            let operation = tokio::time::timeout(Duration::from_secs(590), async {
                if self.local {
                    self.agent.fetch_root_key().await?;
                }
                operation.await
            });
            tokio::select! {
                biased;
                result = operation => result.map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Governance agent call exceeded its deadline",
                    )
                })?,
                error = cancelled => Err(error),
            }
        })
    }
}

impl Probe for AgentProbe {
    fn module_hash(&mut self) -> Result<String> {
        self.call(async {
            let bytes = self
                .agent
                .read_state_canister_module_hash(self.principal)
                .await?;
            Ok(Sha256Digest::from_bytes(
                bytes
                    .try_into()
                    .map_err(|_| invalid("module hash is not SHA-256"))?,
            )
            .to_string())
        })
    }

    fn metadata(&mut self, name: &str) -> Result<Vec<u8>> {
        self.call(async {
            Ok(self
                .agent
                .read_state_canister_metadata(self.principal, name)
                .await?)
        })
    }

    fn report(&mut self, kind: &str) -> Result<Vec<u8>> {
        if !crate::KINDS.contains(&kind) {
            return Err(invalid("unknown Governance report kind"));
        }
        self.call(async {
            Ok(self
                .agent
                .update(&self.principal, "report")
                .with_arg(encode_request(kind)?)
                .call_and_wait()
                .await?)
        })
    }
}
