use super::*;

fn blob(tag: &[u8], value: &[u8]) -> Vec<u8> {
    // Independent literal fixture framing, never a production encoder.
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

fn report(record: &[u8]) -> Result<SubnetSettingsReport, SubnetSettingsError> {
    let mut envelope = vec![16, 172, 2];
    envelope.extend(blob(&[26], record));
    envelope.extend([40, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1]);

    project(
        Principal::from_slice(&[10]),
        &envelope,
        9_007_199_254_740_993,
    )
}

#[test]
fn settings_preserve_absent_empty_and_explicit_values_with_evidence() {
    let absent = report(&[]).unwrap();
    assert_eq!(absent.settings.features, None);
    assert_eq!(absent.settings.resource_limits, None);
    let empty = report(&[0xba, 1, 0, 0x92, 2, 0]).unwrap();
    assert_eq!(empty.settings.features, Some(SubnetFeatures::default()));
    assert_eq!(
        empty.settings.resource_limits,
        Some(ResourceLimits::default())
    );
    let mut record = vec![0xba, 1, 4, 24, 1, 72, 0, 0xc0, 1];
    record.extend([255, 255, 255, 255, 255, 255, 255, 255, 255, 1]);
    record.extend([0x92, 2, 8, 8, 0, 16, 0, 24, 0, 32, 0]);
    let report = report(&record).unwrap();
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["settings"]["max_number_of_canisters"], u64::MAX);
    assert_eq!(json["settings"]["features"]["sev_enabled"], false);
    assert_eq!(
        json["settings"]["resource_limits"]["maximum_query_instructions"],
        0
    );
    assert_eq!(
        report.evidence.source_key,
        format!("subnet_record_{}", report.subnet_id)
    );
    assert_eq!(report.evidence.record_version, 300);
    assert_eq!(report.evidence.record_timestamp_nanoseconds, u64::MAX);
    assert_eq!(
        report.evidence.observed_at_unix_seconds,
        9_007_199_254_740_993
    );
    assert!(report.evidence.query_signature_verified);
    assert!(!report.evidence.result_certified);
    assert!(!report.evidence.tee_attestation_verified);
}

#[test]
fn settings_refuse_missing_content_malformed_fields_and_rejection() {
    let id = Principal::from_slice(&[10]);
    assert!(matches!(
        project(id, &[], 0),
        Err(SubnetSettingsError::MissingContent)
    ));
    assert!(matches!(
        project(id, &[26, 1], 0),
        Err(SubnetSettingsError::Envelope(_))
    ));
    assert!(matches!(
        report(&[0xba, 1, 2, 24, 2]),
        Err(SubnetSettingsError::Settings(wire::Error::Malformed))
    ));
    let error = project(id, &[10, 8, 8, 1, 18, 1, b'x', 26, 1, b'k'], 0).unwrap_err();
    assert!(
        matches!(error, SubnetSettingsError::RegistryRejected { code: 1, reason, key } if reason == "x" && key == b"k")
    );
}

#[test]
fn settings_share_nested_field_budget() {
    let nested = [8, 0].repeat(4095);
    assert!(report(&blob(&[0xba, 1], &nested)).is_ok());
    let nested = [8, 0].repeat(4096);
    assert!(matches!(
        report(&blob(&[0xba, 1], &nested)),
        Err(SubnetSettingsError::Settings(wire::Error::FieldLimit))
    ));
}

#[test]
fn settings_envelope_byte_limit_is_inclusive_and_rejection_wins_over_content() {
    let id = Principal::from_slice(&[10]);
    // Unknown length-delimited root field: the envelope admits exactly 64 KiB.
    // Its contents are opaque, so they do not consume the selected field budget.
    let record = blob(&[10], &vec![0; 65_528]);
    let envelope = blob(&[26], &record);
    assert_eq!(envelope.len(), mainnet::REGISTRY_LIMITS.bytes);
    assert!(project(id, &envelope, 0).is_ok());
    let oversized = blob(&[26], &blob(&[10], &vec![0; 65_529]));
    assert!(matches!(
        project(id, &oversized, 0),
        Err(SubnetSettingsError::Envelope(_))
    ));

    // A Registry rejection must not become a successful default-valued report.
    let mut rejected = vec![10, 2, 8, 1];
    rejected.extend(blob(&[26], &[]));
    assert!(matches!(
        project(id, &rejected, 0),
        Err(SubnetSettingsError::RegistryRejected { code: 1, .. })
    ));
}

#[test]
fn failed_settings_json_retains_category_and_code_without_source_diagnostics() {
    let outcome: SettingsOutcome = Err(SubnetSettingsError::RegistryRejected {
        code: -7,
        reason: "untrusted\u{1b}[31m".into(),
        key: vec![0, 255],
    })
    .into();
    let json = serde_json::to_value(&outcome).unwrap();
    assert_eq!(json["status"], "failed");
    assert_eq!(json["error"]["kind"], "registry_rejected");
    assert_eq!(json["error"]["registry_code"], -7);
    assert!(json["error"].get("reason").is_none());
    assert!(outcome.report().is_none());
    assert!(
        matches!(outcome, SettingsOutcome::Failed { error: SubnetSettingsError::RegistryRejected { reason, key, .. } } if reason.contains('\u{1b}') && key == [0,255])
    );
}
