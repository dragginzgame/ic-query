use super::*;

#[test]
fn get_value_response_reports_large_value_chunk_keys() {
    let response = RegistryGetValueResponse {
        error: None,
        version: 1,
        content: Some(registry_get_value_response::Content::LargeValueChunkKeys(
            proto::LargeValueChunkKeys {
                chunk_content_sha256s: vec![vec![1], vec![2]],
            },
        )),
        timestamp_nanoseconds: 0,
    };

    let content = registry_value_content_from_response("routing_table", response)
        .expect("large value chunk keys");

    match content {
        RegistryValueContent::LargeValueChunkKeys(keys) => {
            assert_eq!(keys.chunk_content_sha256s, vec![vec![1], vec![2]]);
        }
        RegistryValueContent::Value(value) => {
            panic!("expected chunk keys, got inline value {value:?}");
        }
    }
}

#[test]
fn registry_get_chunk_request_candid_round_trips() {
    let request = RegistryGetChunkRequest {
        content_sha256: Some(vec![1, 2, 3]),
    };

    let bytes = Encode!(&request).expect("encode");
    let decoded = Decode!(&bytes, RegistryGetChunkRequest).expect("decode");

    assert_eq!(decoded, request);
}

#[test]
fn validated_chunk_append_concatenates_matching_chunks() {
    let first = b"hello ".to_vec();
    let second = b"world".to_vec();
    let first_hash = sha256_digest(&first);
    let second_hash = sha256_digest(&second);
    let mut value = Vec::new();

    append_validated_chunk(&mut value, &first_hash, first).expect("first chunk");
    append_validated_chunk(&mut value, &second_hash, second).expect("second chunk");

    assert_eq!(value, b"hello world");
}

#[test]
fn validated_chunk_append_rejects_hash_mismatch() {
    let expected = sha256_digest(b"expected");

    let err = append_validated_chunk(&mut Vec::new(), &expected, b"actual".to_vec())
        .expect_err("hash mismatch");

    assert!(matches!(
        err,
        RegistryFetchError::ChunkHashMismatch {
            sha256,
            actual_sha256
        } if sha256 == hex_bytes(&expected)
            && actual_sha256 == hex_bytes(&sha256_digest(b"actual"))
    ));
}

#[test]
fn get_value_response_reports_registry_errors() {
    let response = RegistryGetValueResponse {
        error: Some(proto::RegistryError {
            code: RegistryErrorCode::KeyNotPresent as i32,
            reason: "missing".to_string(),
            key: b"routing_table".to_vec(),
        }),
        version: 1,
        content: None,
        timestamp_nanoseconds: 0,
    };

    let err =
        registry_value_content_from_response("routing_table", response).expect_err("registry");

    assert!(matches!(
        err,
        RegistryFetchError::RegistryValue {
            key,
            code,
            reason
        } if key == "routing_table" && code == "key_not_present" && reason == "missing"
    ));
}
