use crate::tests::{Placement as GetSubnetForCanisterResponseOk, encode_fixture, frame};

use super::{PlacementError, Principal, REPLY_LIMITS, decode_subnet, decoder, report};

fn reply(subnet_id: Option<Principal>) -> Vec<u8> {
    let response: Result<_, String> = Ok(GetSubnetForCanisterResponseOk { subnet_id });
    encode_fixture(&response, 1024).expect("fixture encodes")
}

#[test]
fn replacement_identity_preserves_registry_bytes_and_report_json() {
    for length in 0..=29 {
        let bytes: Vec<_> = (0..length).map(|index| index as u8).collect();
        let identity = Principal::from_slice(&bytes);
        let independent = String::from_utf8(frame(
            "principal-text",
            &serde_json::to_value(&bytes).unwrap(),
        ))
        .unwrap();
        assert_eq!(decode_subnet(&reply(Some(identity))).unwrap(), identity);
        assert_eq!(identity.to_text(), independent);
        assert_eq!(
            serde_json::to_value(identity).unwrap(),
            serde_json::to_value(independent).unwrap()
        );
    }
}

#[test]
fn subnet_identity_survives_decoding_and_json_keeps_raw_evidence() {
    let canister = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let subnet = Principal::from_slice(&[17; 29]);
    let decoded = decode_subnet(&reply(Some(subnet))).unwrap();
    assert_eq!(decoded, subnet);
    let value = serde_json::to_value(report(canister, decoded, 9_007_199_254_740_993)).unwrap();
    assert_eq!(value["schema"], 1);
    assert_eq!(value["canister_id"], canister.to_text());
    assert_eq!(value["subnet_id"], subnet.to_text());
    let evidence = &value["evidence"];
    assert_eq!(
        evidence["observed_at_unix_seconds"].as_u64(),
        Some(9_007_199_254_740_993)
    );
    assert_eq!(evidence["source_method"], "get_subnet_for_canister");
    assert_eq!(evidence["endpoint"], "https://ic0.app");
    assert_eq!(evidence["source_canister"], "rwlgt-iiaaa-aaaaa-aaaaa-cai");
    assert_eq!(evidence["query_signature_verified"], true);
    assert_eq!(evidence["result_certified"], false);
    assert!(evidence["registry_version"].is_null());
    assert_eq!(evidence["selected_lookup_complete"], true);
}

#[test]
fn registry_rejection_and_missing_subnet_are_distinct_failures() {
    assert!(matches!(
        decode_subnet(&reply(None)),
        Err(PlacementError::MissingSubnet)
    ));
    let response: Result<GetSubnetForCanisterResponseOk, _> = Err("unmapped 🌍".to_owned());
    let bytes = encode_fixture(&response, 1024).unwrap();
    assert!(
        matches!(decode_subnet(&bytes), Err(PlacementError::RegistryRejected(message)) if message == "unmapped 🌍")
    );
}

#[test]
fn malformed_trailing_surplus_and_oversized_replies_are_refused() {
    assert!(matches!(
        decode_subnet(b"invalid"),
        Err(PlacementError::Decode(_))
    ));
    let mut trailing = reply(Some(Principal::from_slice(&[1])));
    trailing.push(0);
    assert!(matches!(
        decode_subnet(&trailing),
        Err(PlacementError::Decode(decoder::Error::Trailing))
    ));
    let surplus = frame("placement-surplus", &serde_json::Value::Null);
    assert!(matches!(
        decode_subnet(&surplus),
        Err(PlacementError::Decode(decoder::Error::Arity))
    ));
    assert!(matches!(
        decode_subnet(&vec![0; REPLY_LIMITS.bytes + 1]),
        Err(PlacementError::Decode(decoder::Error::Limit))
    ));
}
