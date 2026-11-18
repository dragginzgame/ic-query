use super::*;

fn read<T>(value: T) -> Observation<T> {
    Observation {
        method: "fixture",
        started_at_unix_ms: Some(1),
        finished_at_unix_ms: Some(2),
        query_signature_verified: true,
        result_certified: false,
        outcome: Outcome::Read { value },
    }
}

#[test]
fn balance_text_escapes_symbol_and_keeps_exact_base_units() {
    let report = BalanceReport {
        schema: 1,
        endpoint: "https://ic0.app",
        ledger: "zfcdd-tqaaa-aaaaq-aaaga-cai".parse().unwrap(),
        account: ic_platform_types::Principal::anonymous().into(),
        balance: read("340282366920938463463374607431768211456".to_owned()),
        decimals: read(8),
        symbol: read("DKP\n\x1b[31m".to_owned()),
        complete: true,
        atomic_snapshot: false,
    };
    let text = balance(&report);
    assert!(text.contains("340282366920938463463374607431768211456"));
    assert!(!text.contains('\x1b'));
    assert!(!text.contains("DKP\n"));
    assert!(text.contains("not certified application state"));
    assert!(text.contains("1 – 2"));
    assert_eq!(
        serde_json::to_value(report).unwrap()["symbol"]["outcome"]["value"],
        "DKP\n\x1b[31m"
    );
}
