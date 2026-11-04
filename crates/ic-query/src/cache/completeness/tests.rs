use super::*;

#[test]
fn completeness_requires_api_exhaustion_and_consistent_page_evidence() {
    for page_size in 0..=2 {
        for page_count in 0..=2 {
            for declared_row_count in 0..=2 {
                for actual_row_count in 0..=2 {
                    let completeness = CacheCollectionCompleteness::api_exhausted(
                        page_size,
                        page_count,
                        declared_row_count,
                        false,
                    );
                    let expected_valid = page_size > 0
                        && page_count > 0
                        && declared_row_count == actual_row_count
                        && actual_row_count <= (page_size * page_count) as usize;

                    assert_eq!(
                        validate_cache_collection_completeness(&completeness, actual_row_count)
                            .is_ok(),
                        expected_valid,
                        "page_size={page_size}, page_count={page_count}, declared_row_count={declared_row_count}, actual_row_count={actual_row_count}"
                    );
                }
            }
        }
    }

    let mut unknown = CacheCollectionCompleteness::api_exhausted(1, 1, 0, false);
    unknown.status = "unknown".to_string();
    assert_eq!(
        validate_cache_collection_completeness(&unknown, 0),
        Err("completeness status is unknown, expected api_exhausted".to_string())
    );
    assert_eq!(
        serde_json::to_value(CacheCollectionCompleteness::api_exhausted(1, 1, 0, false))
            .expect("serialize completeness")["status"],
        "api_exhausted"
    );
}

#[test]
fn completeness_enforces_page_capacity_without_narrow_multiplication() {
    for (page_size, page_count, row_count, valid) in [
        (2, 2, 4, true),
        (2, 2, 5, false),
        (1, 1, 0, true),
        (u32::MAX, 2, 100, true),
        (u32::MAX, u32::MAX, 0, true),
    ] {
        let completeness =
            CacheCollectionCompleteness::api_exhausted(page_size, page_count, row_count, false);
        assert_eq!(
            validate_cache_collection_completeness(&completeness, row_count).is_ok(),
            valid
        );
    }
}
