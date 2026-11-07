use super::*;

fn target() -> IcrcLedgerOptions {
    let matches = super::super::ledger::icrc_token_command()
        .try_get_matches_from([
            "token",
            "aaaaa-aa",
            "--source-endpoint",
            "https://example.com/api",
            "--json",
        ])
        .unwrap();
    IcrcLedgerOptions::from_matches(&matches)
}

#[test]
fn native_ledger_request_retains_explicit_target_time_and_output() {
    let (format, request) = target().into_ledger_request(123);
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(request.source_endpoint, "https://example.com/api");
    assert_eq!(request.ledger_canister_id, "aaaaa-aa");
    assert_eq!(request.now_unix_secs, 123);
}
