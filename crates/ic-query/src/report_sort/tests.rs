use super::*;

#[test]
fn optional_numeric_sort_preserves_missing_keys_and_caller_id_tiebreaks() {
    let original = [
        (Some(2), Some(7)),
        (None, Some(9)),
        (Some(1), None),
        (Some(2), Some(4)),
        (Some(2), None),
    ];
    for (descending, order) in [(false, [2, 3, 0, 4, 1]), (true, [0, 3, 4, 2, 1])] {
        let mut rows = original;
        sort_by_optional_u64(
            &mut rows,
            descending,
            |row| row.0,
            |left, right| compare_optional_ord(left.1, right.1, descending),
        );
        assert_eq!(rows, order.map(|index| original[index]));
    }
}

#[test]
fn text_ordering_folds_ascii_and_preserves_other_bytes() {
    for (left, right, expected) in [
        ("Alpha", "aLPHA", Ordering::Equal),
        ("", "a", Ordering::Less),
        ("A", "aa", Ordering::Less),
        ("beta", "Alpha", Ordering::Greater),
        ("_", "A", Ordering::Less),
        ("Ä", "ä", Ordering::Less),
        ("ÉA", "Éa", Ordering::Equal),
        ("z", "é", Ordering::Less),
    ] {
        assert_eq!(
            compare_ascii_case_insensitive_text(left, right, false),
            expected
        );
        assert_eq!(
            compare_ascii_case_insensitive_text(left, right, true),
            expected.reverse()
        );
    }
    for descending in [false, true] {
        assert_eq!(
            compare_optional_ascii_case_insensitive_text(Some("a"), None, descending),
            Ordering::Less
        );
        assert_eq!(
            compare_optional_ascii_case_insensitive_text(None, Some("a"), descending),
            Ordering::Greater
        );
    }
}
