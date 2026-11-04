use super::{IdentifierMatch, match_identifier};

#[test]
fn exact_matches_win_over_prior_prefixes() {
    assert!(matches!(
        match_identifier(
            ["node-1", "node-2", "node"].into_iter().enumerate(),
            |(_, value)| value,
            Some("node"),
            "node"
        ),
        IdentifierMatch::Exact((2, "node"))
    ));
}

#[test]
fn prefix_matching_preserves_order_and_spelling() {
    assert!(matches!(
        match_identifier(["B", "b"].into_iter(), |value| value, None, "b"),
        IdentifierMatch::Prefix("b")
    ));
    assert!(matches!(
        match_identifier(["b", "a"].into_iter(), |value| value, None, ""),
        IdentifierMatch::Missing
    ));
    match match_identifier(["b2", "b1", "b2"].into_iter(), |value| value, None, "b") {
        IdentifierMatch::Ambiguous(values) => assert_eq!(values, ["b2", "b1", "b2"]),
        _ => panic!("expected ambiguity"),
    }
}
