use super::*;

#[test]
fn budget_rejects_empty_invalid_and_excessive_chunk_references() {
    let mut budget =
        RegistryChunkBudget::new(RegistryChunkLimits::ordinary_value(), 0).expect("empty budget");
    assert!(matches!(
        budget.validated_hashes(&[]),
        Err(RegistryFetchError::EmptyRegistryChunkList)
    ));
    assert!(matches!(
        budget.validated_hashes(&[vec![0; 31]]),
        Err(RegistryFetchError::InvalidRegistryChunkDigest { actual_bytes: 31 })
    ));
    assert!(matches!(
        budget.validated_hashes(&vec![vec![0; 32]; MAX_REGISTRY_CHUNK_REFERENCES + 1]),
        Err(RegistryFetchError::RegistryChunkLimit {
            field: "reference count",
            ..
        })
    ));
}

#[test]
fn budget_hash_verifies_bounds_and_reuses_chunks() {
    let content = b"bounded chunk".to_vec();
    let hash = sha256_digest(&content);
    let mut budget =
        RegistryChunkBudget::new(RegistryChunkLimits::ordinary_value(), 0).expect("empty budget");
    let mut value = Vec::new();

    budget
        .append_fetched_chunk(&mut value, hash, content.clone())
        .expect("verified chunk");
    assert_eq!(value, content);
    assert!(
        budget
            .append_cached_chunk(&mut value, &hash)
            .expect("cached chunk")
    );
    assert_eq!(value, [content.clone(), content].concat());
    assert_eq!(budget.cache.len(), 1);

    let error = budget
        .append_fetched_chunk(&mut Vec::new(), [9; 32], b"wrong".to_vec())
        .expect_err("hash mismatch");
    assert!(matches!(
        error,
        RegistryFetchError::ChunkHashMismatch { .. }
    ));
}

#[test]
fn budget_rejects_chunk_value_total_and_response_overflow() {
    let small_limits = RegistryChunkLimits {
        references: 2,
        chunk_bytes: 3,
        value_bytes: 4,
        total_value_bytes: 5,
        response_bytes: 6,
    };
    let mut budget = RegistryChunkBudget::new(small_limits, 2).expect("small budget");
    let content = b"four".to_vec();
    let error = budget
        .append_fetched_chunk(&mut Vec::new(), sha256_digest(&content), content)
        .expect_err("chunk content limit");
    assert!(matches!(
        error,
        RegistryFetchError::RegistryChunkLimit {
            field: "content bytes",
            ..
        }
    ));

    let value_limits = RegistryChunkLimits {
        chunk_bytes: 3,
        ..small_limits
    };
    let mut value_budget = RegistryChunkBudget::new(value_limits, 0).expect("value budget");
    let mut value = b"two".to_vec();
    let content = b"two".to_vec();
    let error = value_budget
        .append_fetched_chunk(&mut value, sha256_digest(&content), content)
        .expect_err("reconstructed value limit");
    assert!(matches!(
        error,
        RegistryFetchError::RegistryChunkLimit {
            field: "value bytes",
            ..
        }
    ));

    let mut total_budget = RegistryChunkBudget::new(small_limits, 4).expect("total budget");
    let content = b"ok".to_vec();
    let error = total_budget
        .append_fetched_chunk(&mut Vec::new(), sha256_digest(&content), content)
        .expect_err("total reconstructed byte limit");
    assert!(matches!(
        error,
        RegistryFetchError::RegistryChunkLimit {
            field: "reconstructed value bytes",
            ..
        }
    ));

    budget.record_query_response(6).expect("response boundary");
    let error = budget
        .record_query_response(1)
        .expect_err("response total limit");
    assert!(matches!(
        error,
        RegistryFetchError::RegistryChunkLimit {
            field: "response bytes",
            ..
        }
    ));
    assert_eq!(budget.query_call_count, 1);
    assert_eq!(budget.response_bytes, 6);
    budget
        .record_query_response(0)
        .expect("remaining query allowance");
    assert!(matches!(
        budget.record_query_response(0),
        Err(RegistryFetchError::RegistryChunkLimit {
            field: "query call count",
            ..
        })
    ));
    assert_eq!(budget.query_call_count, 2);
    assert_eq!(budget.response_bytes, 6);
}
