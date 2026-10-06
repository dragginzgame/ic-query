//! Development-only admission of Governance probe artifacts and ICP responses.

use ic_host_tools::{
    artifact::{self, Sha256Digest},
    response::{self, ResponseFormat, ResponseLimits},
    wasm::{self, InspectionLimits},
};
use std::{
    env,
    error::Error,
    io::{self, Write},
    path::Path,
};

const WASM_BYTES: usize = 64 * 1024 * 1024;
const RESPONSE_INPUT_BYTES: usize = 8 * 1024 * 1024;
const RESPONSE_DECODED_BYTES: usize = 2 * 1024 * 1024;

fn inspect_wasm(path: &Path) -> Result<Sha256Digest, Box<dyn Error>> {
    let bytes = artifact::read_file(path, WASM_BYTES)?;
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

fn decode_response(input: &[u8]) -> Result<String, Box<dyn Error>> {
    let bytes = response::decode(
        input,
        ResponseFormat::Json,
        ResponseLimits {
            input_bytes: RESPONSE_INPUT_BYTES,
            decoded_bytes: RESPONSE_DECODED_BYTES,
        },
    )?;
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "probe must return exactly one Candid text value",
        )
    };
    let mut decoder = candid::de::IDLDeserialize::new(&bytes).map_err(|_| invalid())?;
    let text = decoder.get_value::<String>().map_err(|_| invalid())?;
    if !decoder.is_done() {
        return Err(invalid().into());
    }
    decoder.done().map_err(|_| invalid())?;
    Ok(text)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let command = args.next().and_then(|argument| argument.into_string().ok());
    let path = args.next();
    if args.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "too many arguments").into());
    }
    let mut output = io::stdout().lock();
    match (command.as_deref(), path) {
        (Some("inspect-wasm"), Some(path)) => {
            writeln!(output, "{}", inspect_wasm(Path::new(&path))?)?;
        }
        (Some("decode-response"), None) => {
            let input = artifact::read_reader(io::stdin().lock(), RESPONSE_INPUT_BYTES)?;
            output.write_all(decode_response(&input)?.as_bytes())?;
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected inspect-wasm <path> or decode-response with JSON stdin",
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_host_tools::response::ResponseError;
    use std::fmt::Write as _;

    fn envelope(bytes: &[u8]) -> Vec<u8> {
        let mut hex = String::new();
        for byte in bytes {
            write!(hex, "{byte:02x}").expect("fixture hex");
        }
        serde_json::to_vec(&serde_json::json!({"response_bytes": hex})).expect("fixture JSON")
    }

    #[test]
    fn preserves_the_report_text_without_numeric_conversion() {
        let text = r#"{"raw_amount":184467440737095516160,"status":"ok"}"#;
        let bytes = candid::encode_one(text).expect("Candid text");
        assert_eq!(
            decode_response(&envelope(&bytes)).expect("report text"),
            text
        );
    }

    #[test]
    fn rejects_wrong_types_extra_values_and_trailing_bytes() {
        let wrong = candid::encode_one(7_u64).expect("Candid number");
        let extra = candid::encode_args(("{}", "{}")).expect("two Candid values");
        let mut trailing = candid::encode_one("{}").expect("Candid text");
        trailing.push(0);
        for bytes in [wrong, extra, trailing] {
            let error = decode_response(&envelope(&bytes)).expect_err("not one text value");
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
    fn admits_the_original_envelope_before_candid_decoding() {
        let error = decode_response(br#"{"response_bytes":"","response_bytes":""}"#)
            .expect_err("ambiguous response");
        assert!(matches!(
            error.downcast_ref::<ResponseError>(),
            Some(ResponseError::Json { .. })
        ));
    }
}
