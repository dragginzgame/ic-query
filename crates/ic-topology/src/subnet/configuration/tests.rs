use super::*;

fn blob(tag: u8, value: &[u8]) -> Vec<u8> {
    // Independent length-delimited wire fixture, not a production codec.
    let mut bytes = vec![tag];
    let mut length = value.len();
    while length >= 128 {
        bytes.push((length as u8 & 127) | 128);
        length >>= 7;
    }
    bytes.push(length as u8);
    bytes.extend_from_slice(value);
    bytes
}

fn envelope(record: &[u8]) -> Vec<u8> {
    let mut bytes = vec![16, 172, 2]; // Record mutation version 300.
    bytes.extend(blob(26, record));
    bytes.extend([40, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1]);
    bytes
}

#[test]
fn configuration_preserves_selected_fields_raw_strings_and_record_evidence() {
    let subnet = Principal::from_slice(&[10]);
    let version = "version\n\u{1b}[31m型";
    let mut record = vec![26, 1, 9, 26, 1, 1];
    record.extend(blob(74, version.as_bytes()));
    // Unknown signed enum -1, both halt flags true.
    record.extend([
        120, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 136, 1, 1, 224, 1, 1,
    ]);
    let report = project(subnet, &envelope(&record), 9_007_199_254_740_993).unwrap();
    assert_eq!(
        report.registered_node_ids,
        [1, 9].map(|id| Principal::from_slice(&[id]))
    );
    assert_eq!(report.replica_version_id, version);
    assert_eq!(report.registered_subnet_type, -1);
    assert!(report.is_halted && report.halt_at_cup_height);
    let json = serde_json::to_value(report).unwrap();
    assert_eq!(json["schema"], 1);
    assert_eq!(json["subnet_id"], subnet.to_text());
    assert_eq!(json["replica_version_id"], version);
    assert_eq!(json["registered_subnet_type"], -1);
    assert_eq!(
        json["evidence"]["source_key"],
        format!("subnet_record_{subnet}")
    );
    assert_eq!(json["evidence"]["record_version"], 300);
    assert_eq!(json["evidence"]["record_timestamp_nanoseconds"], u64::MAX);
    assert_eq!(
        json["evidence"]["observed_at_unix_seconds"],
        9_007_199_254_740_993_u64
    );
    assert_eq!(json["evidence"]["query_signature_verified"], true);
    assert_eq!(json["evidence"]["result_certified"], false);
    assert_eq!(json["evidence"]["membership_complete"], true);
    assert!(json["evidence"].get("registry_version").is_none());
}

#[test]
fn present_default_and_cloud_engine_records_do_not_invent_an_effective_version() {
    let subnet = Principal::from_slice(&[10]);
    for (record, expected_type) in [(&[][..], 0), (&[120, 5][..], 5)] {
        let report = project(subnet, &envelope(record), 0).unwrap();
        assert!(report.registered_node_ids.is_empty());
        assert!(report.replica_version_id.is_empty());
        assert_eq!(report.registered_subnet_type, expected_type);
        assert!(!report.is_halted && !report.halt_at_cup_height);
        assert!(report.evidence.membership_complete);
    }
    assert!(matches!(
        project(subnet, &[], 0),
        Err(SubnetConfigurationError::MissingContent)
    ));
}

#[test]
fn version_budget_counts_utf8_bytes_without_truncation() {
    let subnet = Principal::from_slice(&[10]);
    let exact = "é".repeat(MAX_VERSION_BYTES / 2);
    let record = blob(74, exact.as_bytes());
    assert_eq!(
        project(subnet, &envelope(&record), 0)
            .unwrap()
            .replica_version_id,
        exact
    );
    let excess = blob(74, format!("{exact}x").as_bytes());
    assert!(matches!(
        project(subnet, &envelope(&excess), 0),
        Err(SubnetConfigurationError::VersionLimit)
    ));
}

#[test]
fn invalid_duplicate_and_excess_members_are_refused_but_exact_limit_is_retained() {
    let subnet = Principal::from_slice(&[10]);
    assert!(matches!(
        project(subnet, &envelope(&[26, 1, 1, 26, 1, 1]), 0),
        Err(SubnetConfigurationError::DuplicateIdentity)
    ));
    assert!(matches!(
        project(subnet, &envelope(&blob(26, &[0; 30])), 0),
        Err(SubnetConfigurationError::InvalidIdentity { index: 0, .. })
    ));
    let mut record = Vec::new();
    for id in 0..LIMITS.items {
        record.extend(blob(26, &id.to_be_bytes()));
    }
    assert_eq!(
        project(subnet, &envelope(&record), 0)
            .unwrap()
            .registered_node_ids
            .len(),
        LIMITS.items
    );
    record.extend(blob(26, &LIMITS.items.to_be_bytes()));
    assert!(matches!(
        project(subnet, &envelope(&record), 0),
        Err(SubnetConfigurationError::Codec(protobuf::Error::ItemLimit))
    ));
}

#[test]
fn supplier_refusals_and_registry_causes_remain_typed() {
    let subnet = Principal::from_slice(&[10]);
    for (bytes, expected) in [
        (vec![34, 0], protobuf::Error::ChunkedValue),
        (vec![26, 3, 18], protobuf::Error::Malformed),
        (vec![0; LIMITS.bytes + 1], protobuf::Error::ByteLimit),
        (envelope(&[74, 1, 255]), protobuf::Error::Malformed),
    ] {
        assert!(
            matches!(project(subnet, &bytes, 0), Err(SubnetConfigurationError::Codec(error)) if error == expected)
        );
    }
    let error = project(subnet, &[10, 9, 8, 231, 7, 18, 1, 27, 26, 1, b'k'], 0).unwrap_err();
    assert!(!error.to_string().contains('\u{1b}'));
    assert!(
        matches!(error, SubnetConfigurationError::RegistryRejected { code: 999, reason, key } if reason == "\u{1b}" && key == b"k")
    );
}
