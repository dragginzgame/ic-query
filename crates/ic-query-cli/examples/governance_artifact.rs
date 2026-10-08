//! Development-only Governance agent calls, artifact admission and receipt publication.

use candid::Principal;
use ic_agent::Agent;
use ic_host_artifacts::{
    artifact::Sha256Digest,
    wasm::{self, InspectionLimits},
};
use ic_host_fs::{
    durable::{create_private_bytes_with_parents, write_with},
    read::read_file,
};
use std::{
    env,
    error::Error,
    ffi::OsString,
    io::{self, Write},
    path::Path,
    time::Duration,
};
use url::Url;

const WASM_BYTES: usize = 64 * 1024 * 1024;
const RESPONSE_BODY_BYTES: usize = 8 * 1024 * 1024;
const REPLY_BYTES: usize = 2 * 1024 * 1024;
const USAGE: &str = "Governance smoke helper\n\nCommands:\n  help\n  inspect-wasm <path>\n  metadata <local|mainnet-smoke> <endpoint> <principal> <name>\n  module-hash <local|mainnet-smoke> <endpoint> <principal>\n  report <local|mainnet-smoke> <endpoint> <principal> <kind> <reply-file>\n  write-receipt <path>";

fn write_receipt(path: &Path, mut input: impl io::Read) -> io::Result<()> {
    write_with(path, |file| io::copy(&mut input, file))?;
    Ok(())
}

fn inspect_wasm(path: &Path) -> Result<Sha256Digest, Box<dyn Error>> {
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

fn decode_reply(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    if bytes.len() > REPLY_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "probe reply exceeds 2 MiB").into());
    }
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "probe must return exactly one Candid text value",
        )
    };
    let mut decoder = candid::de::IDLDeserialize::new(bytes).map_err(|_| invalid())?;
    let text = decoder.get_value::<String>().map_err(|_| invalid())?;
    if !decoder.is_done() {
        return Err(invalid().into());
    }
    decoder.done().map_err(|_| invalid())?;
    Ok(text)
}

fn publish_reply(path: &Path, bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    // Retain the exact reply before Candid admission, without replacing evidence.
    create_private_bytes_with_parents(path, bytes)?;
    decode_reply(bytes)
}

fn text(value: &OsString) -> io::Result<&str> {
    value
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "expected UTF-8 argument"))
}

fn agent(environment: &str, endpoint: &str) -> Result<Agent, Box<dyn Error>> {
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
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "endpoint does not match the selected network trust policy",
        )
        .into());
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
        .with_max_response_body_size(RESPONSE_BODY_BYTES)
        .build()?)
}

fn agent_call(command: &str, args: &[OsString]) -> Result<Vec<u8>, Box<dyn Error>> {
    let (environment, endpoint, principal, rest) = match args {
        [environment, endpoint, principal, rest @ ..] => (
            text(environment)?,
            text(endpoint)?,
            Principal::from_text(text(principal)?)?,
            rest,
        ),
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into()),
    };
    match (command, rest) {
        ("module-hash", []) => {}
        ("metadata", [name]) => {
            text(name)?;
        }
        ("report", [kind, _])
            if matches!(
                text(kind)?,
                "economics" | "metrics" | "reward_event" | "maturity_modulation"
            ) => {}
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into()),
    }
    let agent = agent(environment, endpoint)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(590), async {
            if environment == "local" {
                agent.fetch_root_key().await?;
            }
            match (command, rest) {
                ("module-hash", []) => {
                    let bytes = agent.read_state_canister_module_hash(principal).await?;
                    let digest = Sha256Digest::from_bytes(bytes.try_into().map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "module hash is not SHA-256")
                    })?);
                    Ok(digest.to_string().into_bytes())
                }
                ("metadata", [name]) => Ok(agent
                    .read_state_canister_metadata(principal, text(name)?)
                    .await?),
                ("report", [kind, path]) => {
                    let kind = text(kind)?;
                    let bytes = agent
                        .update(&principal, "report")
                        .with_arg(candid::encode_one(kind)?)
                        .call_and_wait()
                        .await?;
                    Ok(publish_reply(Path::new(path), &bytes)?.into_bytes())
                }
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into()),
            }
        })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "Governance agent call exceeded its deadline",
            )
        })?
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let command = args.first().map(text).transpose()?;
    let rest = args.get(1..).unwrap_or_default();
    let mut output = io::stdout().lock();
    match (command, rest) {
        (None | Some("help"), []) => writeln!(output, "{USAGE}")?,
        (Some("inspect-wasm"), [path]) => {
            writeln!(output, "{}", inspect_wasm(Path::new(&path))?)?;
        }
        (Some(command @ ("metadata" | "module-hash" | "report")), args) => {
            output.write_all(&agent_call(command, args)?)?;
        }
        (Some("write-receipt"), [path]) => {
            write_receipt(Path::new(&path), io::stdin().lock())?;
        }
        _ => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read as _;

    #[test]
    fn receipt_stream_failure_preserves_the_previous_complete_bytes() {
        struct FailingReader;

        impl io::Read for FailingReader {
            fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::from(io::ErrorKind::BrokenPipe))
            }
        }

        let directory = env::temp_dir().join(format!(
            "ic-query-receipt-stream-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("receipt.json");
        let bytes = br#"{"schema_version":1,"status":"running"}
"#;
        write_receipt(&path, bytes.as_slice()).expect("publish complete receipt");
        let error = write_receipt(&path, io::repeat(b'x').take(20_000).chain(FailingReader))
            .expect_err("failed source cannot publish a partial receipt");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn preserves_the_report_text_without_numeric_conversion() {
        let text = r#"{"raw_amount":184467440737095516160,"status":"ok"}"#;
        let bytes = candid::encode_one(text).expect("Candid text");
        assert_eq!(decode_reply(&bytes).expect("report text"), text);
    }

    #[test]
    fn rejects_wrong_types_extra_values_and_trailing_bytes() {
        let wrong = candid::encode_one(7_u64).expect("Candid number");
        let extra = candid::encode_args(("{}", "{}")).expect("two Candid values");
        let mut trailing = candid::encode_one("{}").expect("Candid text");
        trailing.push(0);
        let mut truncated = candid::encode_one("payload").expect("Candid text");
        truncated.pop();
        for bytes in [wrong, extra, trailing, truncated] {
            let error = decode_reply(&bytes).expect_err("not one text value");
            assert_eq!(
                error
                    .downcast_ref::<io::Error>()
                    .expect("typed Candid failure")
                    .kind(),
                io::ErrorKind::InvalidData
            );
        }
    }

    #[test]
    fn reply_admission_retains_exact_private_bytes_without_replacing_evidence() {
        let directory = env::temp_dir().join(format!(
            "ic-query-agent-reply-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        for (name, bytes) in [
            ("invalid.candid", b"not Candid".to_vec()),
            ("oversized.candid", vec![0; REPLY_BYTES + 1]),
        ] {
            let path = directory.join(name);
            assert!(publish_reply(&path, &bytes).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            assert!(publish_reply(&path, &candid::encode_one("{}").unwrap()).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;
                assert_eq!(
                    std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
        }
        let path = directory.join("valid.candid");
        let bytes = candid::encode_one("{}\n").unwrap();
        assert_eq!(publish_reply(&path, &bytes).unwrap(), "{}\n");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn invalid_operation_is_refused_before_network_io() {
        let args = [
            "local",
            "http://127.0.0.1:1/",
            "aaaaa-aa",
            "unknown",
            "reply.candid",
        ]
        .map(OsString::from);
        let error = agent_call("report", &args).unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn trust_policy_refuses_remote_local_roots_and_unclean_endpoints() {
        for (environment, endpoint) in [
            ("local", "http://example.com"),
            ("local", "https://localhost"),
            ("mainnet-smoke", "http://example.com"),
            ("mainnet-smoke", "https://user:secret@example.com"),
            ("local", "http://127.0.0.1/?network=other"),
        ] {
            assert!(agent(environment, endpoint).is_err());
        }
        assert!(agent("local", "http://127.0.0.1:1234/").is_ok());
        assert!(agent("local", "http://[::1]:1234/").is_ok());
        assert!(agent("mainnet-smoke", "https://icp-api.io/").is_ok());
    }
}
