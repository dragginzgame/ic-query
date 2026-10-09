//! Direct IC protocol calls and bounded admission of original reply evidence.

use crate::{Result, command, invalid};
use candid::{
    Principal,
    de::{DecoderConfig, IDLDeserialize},
};
use futures::future::{Either, select};
use ic_agent::Agent;
use ic_host_artifacts::{
    artifact::Sha256Digest,
    wasm::{self, InspectionLimits},
};
use ic_host_fs::{durable::create_private_bytes_with_parents, read::read_file};
use std::{future::Future, io, path::Path, time::Duration};
use url::Url;

pub const WASM_BYTES: usize = 64 * 1024 * 1024;
const REPLY_BYTES: usize = 2 * 1024 * 1024;

pub fn inspect_wasm(path: &Path) -> Result<Sha256Digest> {
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
    Ok(Sha256Digest::compute(&bytes))
}

pub fn decode_reply(bytes: &[u8]) -> Result<String> {
    if bytes.len() > REPLY_BYTES {
        return Err(invalid("probe reply exceeds 2 MiB"));
    }
    let quota = bytes
        .len()
        .saturating_mul(32)
        .saturating_add(1_000_000)
        .min(256_000_000);
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(quota)
        .set_skipping_quota(100_000)
        .set_max_type_len(4_096)
        .set_max_header_len(64 * 1_024);
    let invalid_reply =
        || invalid("probe must return exactly one Candid text value within decoding limits");
    let mut decoder =
        IDLDeserialize::new_with_config(bytes, &config).map_err(|_| invalid_reply())?;
    let text = decoder.get_value::<String>().map_err(|_| invalid_reply())?;
    if !decoder.is_done() {
        return Err(invalid_reply());
    }
    decoder.done().map_err(|_| invalid_reply())?;
    Ok(text)
}

pub fn publish_reply(path: &Path, bytes: &[u8]) -> Result<String> {
    // Evidence is private and create-new, including replies rejected by admission.
    create_private_bytes_with_parents(path, bytes)?;
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
            let cancelled = Box::pin(async {
                loop {
                    if let Some(signal) = command::interruption() {
                        return invalid(&format!(
                            "interrupted by {}",
                            command::signal_name(signal)
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            });
            let operation = Box::pin(tokio::time::timeout(Duration::from_secs(590), async {
                if self.local {
                    self.agent.fetch_root_key().await?;
                }
                operation.await
            }));
            match select(operation, cancelled).await {
                Either::Left((result, _)) => result.map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Governance agent call exceeded its deadline",
                    )
                })?,
                Either::Right((error, _)) => Err(error),
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
                .with_arg(candid::encode_one(kind)?)
                .call_and_wait()
                .await?)
        })
    }
}
