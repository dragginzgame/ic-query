use super::*;

fn envelope(record: &[u8]) -> Vec<u8> {
    // Independently assembled protobuf envelope: version 300, inline content,
    // mutation timestamp u64::MAX. These bytes exercise the public supplier codec.
    let mut bytes = vec![16, 172, 2, 26];
    let mut length = record.len();
    while length >= 128 {
        bytes.push((length as u8 & 127) | 128);
        length >>= 7;
    }
    bytes.push(length as u8);
    bytes.extend_from_slice(record);
    bytes.extend_from_slice(&[40, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1]);
    bytes
}

#[test]
fn inventory_sorts_every_identity_and_preserves_raw_record_evidence() {
    let report = project(&envelope(&[18, 1, 9, 18, 1, 1, 18, 1, 4]), 123).unwrap();
    assert_eq!(
        report.subnet_ids,
        [1, 4, 9].map(|id| Principal::from_slice(&[id]))
    );
    assert!(report.evidence.selected_record_complete);
    assert!(report.evidence.query_signature_verified);
    assert!(!report.evidence.result_certified);
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["schema"], 1);
    assert_eq!(json["evidence"]["record_version"], 300);
    assert_eq!(json["evidence"]["record_timestamp_nanoseconds"], u64::MAX);
    assert_eq!(json["evidence"]["observed_at_unix_seconds"], 123);
    assert_eq!(json["evidence"]["source_key"], "subnet_list");
    assert!(json["evidence"].get("registry_version").is_none());
}

#[test]
fn incomplete_ambiguous_and_invalid_records_never_become_inventory() {
    assert!(matches!(
        project(&[], 0),
        Err(SubnetListError::MissingContent)
    ));
    assert!(matches!(
        project(&envelope(&[]), 0),
        Err(SubnetListError::EmptyList)
    ));
    assert!(matches!(
        project(&envelope(&[18, 1, 4, 18, 1, 4]), 0),
        Err(SubnetListError::DuplicateIdentity)
    ));
    let mut invalid = vec![18, 30];
    invalid.extend_from_slice(&[0; 30]);
    assert!(matches!(
        project(&envelope(&invalid), 0),
        Err(SubnetListError::InvalidIdentity { index: 0, .. })
    ));
    assert!(matches!(
        project(&[34, 0], 0),
        Err(SubnetListError::Codec(protobuf::Error::ChunkedValue))
    ));
    assert!(matches!(
        project(&[26, 3, 18], 0),
        Err(SubnetListError::Codec(protobuf::Error::Malformed))
    ));
    let oversized = vec![0; LIMITS.bytes + 1];
    assert!(matches!(
        project(&oversized, 0),
        Err(SubnetListError::Codec(protobuf::Error::ByteLimit))
    ));
    let too_many = [18, 0].repeat(LIMITS.items + 1);
    assert!(matches!(
        project(&envelope(&too_many), 0),
        Err(SubnetListError::Codec(protobuf::Error::ItemLimit))
    ));
}

#[test]
fn registry_rejection_retains_structured_cause_without_printing_source_text() {
    let error = project(&[10, 9, 8, 231, 7, 18, 1, 27, 26, 1, b'k'], 0).unwrap_err();
    assert!(!error.to_string().contains('\u{1b}'));
    assert!(
        matches!(error, SubnetListError::RegistryRejected { code: 999, reason, key }
        if reason == "\u{1b}" && key == b"k")
    );
}
