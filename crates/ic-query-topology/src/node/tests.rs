use super::*;

fn blob(tag: &[u8], value: &[u8]) -> Vec<u8> {
    // Independently authored wire fixtures for the selected schema fields.
    let mut bytes = tag.to_vec();
    let mut length = value.len();
    while length >= 128 {
        bytes.push((length as u8 & 127) | 128);
        length >>= 7;
    }
    bytes.push(length as u8);
    bytes.extend_from_slice(value);
    bytes
}

fn record(chip: Option<&[u8]>, hostos: Option<&str>) -> Vec<u8> {
    let mut bytes = vec![0x7a, 1, 1]; // Field 15: operator principal bytes [1].
    if let Some(chip) = chip {
        bytes.extend(blob(&[0x82, 1], chip)); // Optional field 16.
    }
    if let Some(hostos) = hostos {
        bytes.extend(blob(&[0x8a, 1], hostos.as_bytes())); // Optional field 17.
    }
    bytes
}

fn envelope(record: &[u8]) -> Vec<u8> {
    let mut bytes = vec![16, 172, 2]; // Record mutation version 300.
    bytes.extend(blob(&[26], record));
    bytes.extend([40, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1]);
    bytes
}

#[test]
fn node_projection_preserves_raw_fields_and_limits_its_evidence_claims() {
    let node = Principal::from_slice(&[10]);
    let version = "host\n\u{1b}[31m型";
    let bytes = envelope(&record(Some(&[0, 255, 27]), Some(version)));
    let report = project(node, &bytes, 9_007_199_254_740_993).unwrap();
    assert_eq!(report.node_operator_id, Principal::from_slice(&[1]));
    assert_eq!(report.chip_id.as_deref(), Some(&[0, 255, 27][..]));
    assert_eq!(report.hostos_version_id.as_deref(), Some(version));
    let json = serde_json::to_value(report).unwrap();
    assert_eq!(json["schema"], 1);
    assert_eq!(json["node_id"], node.to_text());
    assert_eq!(
        json["node_operator_id"],
        Principal::from_slice(&[1]).to_text()
    );
    assert_eq!(json["chip_id"], serde_json::json!([0, 255, 27]));
    assert_eq!(json["hostos_version_id"], version);
    assert_eq!(
        json["evidence"]["source_key"],
        format!("node_record_{node}")
    );
    assert_eq!(json["evidence"]["endpoint"], "https://ic0.app");
    assert_eq!(
        json["evidence"]["source_canister"],
        "rwlgt-iiaaa-aaaaa-aaaaa-cai"
    );
    assert_eq!(json["evidence"]["source_method"], "get_value");
    assert_eq!(json["evidence"]["record_version"], 300);
    assert_eq!(json["evidence"]["record_timestamp_nanoseconds"], u64::MAX);
    assert_eq!(
        json["evidence"]["observed_at_unix_seconds"],
        9_007_199_254_740_993_u64
    );
    assert_eq!(json["evidence"]["query_signature_verified"], true);
    assert_eq!(json["evidence"]["result_certified"], false);
    assert_eq!(json["evidence"]["tee_attestation_verified"], false);
}

#[test]
fn optional_fields_preserve_absent_empty_and_nonempty_values_independently() {
    let node = Principal::from_slice(&[10]);
    for chip in [None, Some(&[][..]), Some(&[0][..])] {
        for hostos in [None, Some(""), Some("configured")] {
            let report = project(node, &envelope(&record(chip, hostos)), 0).unwrap();
            assert_eq!(report.chip_id.as_deref(), chip);
            assert_eq!(report.hostos_version_id.as_deref(), hostos);
            let json = serde_json::to_value(report).unwrap();
            assert_eq!(json["chip_id"], serde_json::to_value(chip).unwrap());
            assert_eq!(
                json["hostos_version_id"],
                serde_json::to_value(hostos).unwrap()
            );
        }
    }
}

#[test]
fn missing_content_and_missing_or_invalid_operator_remain_distinct() {
    let node = Principal::from_slice(&[10]);
    assert!(matches!(
        project(node, &[], 0),
        Err(NodeError::MissingContent)
    ));
    for record in [&[][..], &[0x7a, 0][..]] {
        assert!(matches!(
            project(node, &envelope(record), 0),
            Err(NodeError::MissingOperator)
        ));
    }
    assert!(matches!(
        project(node, &envelope(&blob(&[0x7a], &[0; 30])), 0),
        Err(NodeError::InvalidOperator(_))
    ));
}

#[test]
fn independent_payload_budgets_accept_exact_limits_and_refuse_excess() {
    let node = Principal::from_slice(&[10]);
    let chip = vec![255; MAX_CHIP_ID_BYTES];
    let hostos = "é".repeat(MAX_HOSTOS_VERSION_BYTES / 2);
    let report = project(node, &envelope(&record(Some(&chip), Some(&hostos))), 0).unwrap();
    assert_eq!(report.chip_id, Some(chip));
    assert_eq!(report.hostos_version_id, Some(hostos.clone()));
    assert!(matches!(
        project(
            node,
            &envelope(&record(Some(&[0; MAX_CHIP_ID_BYTES + 1]), None)),
            0
        ),
        Err(NodeError::ChipIdLimit)
    ));
    assert!(matches!(
        project(
            node,
            &envelope(&record(None, Some(&format!("{hostos}x")))),
            0
        ),
        Err(NodeError::HostosVersionLimit)
    ));
}

#[test]
fn supplier_and_registry_failures_stay_typed_without_terminal_source_text() {
    let node = Principal::from_slice(&[10]);
    for (bytes, expected) in [
        (vec![34, 0], protobuf::Error::ChunkedValue),
        (vec![26, 3, 18], protobuf::Error::Malformed),
        (vec![0; LIMITS.bytes + 1], protobuf::Error::ByteLimit),
        (
            envelope(&[0x7a, 1, 1, 0x8a, 1, 1, 255]),
            protobuf::Error::Malformed,
        ),
    ] {
        assert!(
            matches!(project(node, &bytes, 0), Err(NodeError::Codec(error)) if error == expected)
        );
    }
    let error = project(node, &[10, 9, 8, 231, 7, 18, 1, 27, 26, 1, b'k'], 0).unwrap_err();
    assert!(!error.to_string().contains('\u{1b}'));
    assert!(
        matches!(error, NodeError::RegistryRejected { code: 999, reason, key } if reason == "\u{1b}" && key == b"k")
    );
}
