use super::{IcHostError, validate_offset_page};

#[test]
fn offset_pages_return_a_hint_only_for_nonempty_unfinished_pages() {
    for (offset, total, returned, next) in [
        (0, 5, 2, Some(2)),
        (2, 5, 2, Some(4)),
        (3, 5, 2, None),
        (0, 0, 0, None),
        (5, 5, 0, None),
        (u64::MAX, 0, 0, None),
        (u64::MAX - 1, u64::MAX, 1, None),
    ] {
        assert_eq!(
            validate_offset_page(offset, total, returned, "total", "offset overflow")
                .expect("valid page metadata"),
            next,
        );
    }
}

#[test]
fn inconsistent_or_overflowing_offset_pages_reject_source_data() {
    for (offset, total, returned) in [
        (0, 1, 2),
        (5, 5, 1),
        (6, 5, 1),
        (4, 5, 2),
        (u64::MAX - 1, u64::MAX, 2),
    ] {
        assert!(matches!(
            validate_offset_page(offset, total, returned, "total", "offset overflow"),
            Err(IcHostError::InvalidSourceData { .. })
        ));
    }
}
