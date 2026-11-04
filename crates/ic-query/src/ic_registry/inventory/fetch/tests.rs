use super::*;

#[test]
fn required_node_key_not_present_becomes_missing_node_relation() {
    let node_principal = "node-a";
    let error = RegistryFetchError::RegistryValue {
        key: node_record_key(node_principal),
        code: "key_not_present".to_string(),
        reason: "missing at requested version".to_string(),
    };

    let translated = translate_missing_node_record(node_principal, error);

    assert!(matches!(
        translated,
        RegistryFetchError::MissingNodeRecord { node_principal: found }
            if found == node_principal
    ));
}

#[test]
fn required_operator_key_not_present_keeps_all_referencing_nodes() {
    let node_operator_principal = "operator-a";
    let referencing_node_principals = vec!["node-a".to_string(), "node-b".to_string()];
    let error = RegistryFetchError::RegistryValue {
        key: node_operator_record_key(node_operator_principal),
        code: "key_not_present".to_string(),
        reason: "missing at requested version".to_string(),
    };

    let translated = translate_missing_node_operator_record(
        node_operator_principal,
        &referencing_node_principals,
        error,
    );

    assert!(matches!(
        translated,
        RegistryFetchError::MissingNodeOperatorRecord {
            node_operator_principal: found_operator,
            referencing_node_principals: found_nodes,
        } if found_operator == node_operator_principal
            && found_nodes == referencing_node_principals
    ));
}

#[test]
fn unrelated_registry_value_error_is_preserved() {
    let error = RegistryFetchError::RegistryValue {
        key: node_record_key("node-a"),
        code: "version_not_latest".to_string(),
        reason: "version was compacted".to_string(),
    };

    let translated = translate_missing_node_record("node-a", error);

    assert!(matches!(
        translated,
        RegistryFetchError::RegistryValue { key, code, reason }
            if key == node_record_key("node-a")
                && code == "version_not_latest"
                && reason == "version was compacted"
    ));
}

#[test]
fn node_relation_admission_rejects_missing_and_invalid_operator_ids() {
    let missing = BTreeMap::from([(
        "node-a".to_string(),
        NodeRecord {
            node_operator_id: Vec::new(),
        },
    )]);
    assert!(matches!(
        node_operator_principals_from_records(missing),
        Err(RegistryFetchError::MissingNodeOperatorPrincipal { node_principal })
            if node_principal == "node-a"
    ));
    let invalid = BTreeMap::from([(
        "node-b".to_string(),
        NodeRecord {
            node_operator_id: vec![1; 30],
        },
    )]);
    assert!(matches!(
        node_operator_principals_from_records(invalid),
        Err(RegistryFetchError::InvalidPrincipal {
            field: "node_record.node_operator_id",
            ..
        })
    ));
}

#[test]
fn operator_reference_context_is_canonically_ordered() {
    let node_a = Principal::self_authenticating(b"node-a").to_text();
    let node_b = Principal::self_authenticating(b"node-b").to_text();
    let node_operator = Principal::self_authenticating(b"operator").to_text();
    let node_operator_id = Principal::from_text(&node_operator)
        .expect("operator principal")
        .as_slice()
        .to_vec();
    let records = BTreeMap::from([
        (
            node_b.clone(),
            NodeRecord {
                node_operator_id: node_operator_id.clone(),
            },
        ),
        (node_a.clone(), NodeRecord { node_operator_id }),
    ]);

    let references = node_operator_references(
        &node_operator_principals_from_records(records).expect("node operator principals"),
    );

    assert_eq!(references.get(&node_operator), Some(&vec![node_a, node_b]));
}
