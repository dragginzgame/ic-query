//! Match caller-normalized identifiers; callers own ordering, deduplication and diagnostics.

///
/// IdentifierMatch
///
/// Exact or prefix selection retaining caller items and ambiguity order.
///

pub enum IdentifierMatch<Item> {
    Exact(Item),
    Prefix(Item),
    Missing,
    Ambiguous(Vec<Item>),
}

pub fn match_identifier<Item>(
    identifiers: impl Iterator<Item = Item>,
    text: impl Fn(&Item) -> &str,
    exact: Option<&str>,
    prefix: &str,
) -> IdentifierMatch<Item> {
    let mut matches = Vec::new();
    for identifier in identifiers {
        let candidate = text(&identifier);
        if Some(candidate) == exact {
            return IdentifierMatch::Exact(identifier);
        }
        if !prefix.is_empty() && candidate.starts_with(prefix) {
            matches.push(identifier);
        }
    }
    match matches.len() {
        0 => IdentifierMatch::Missing,
        1 => IdentifierMatch::Prefix(matches.pop().expect("one prefix match")),
        _ => IdentifierMatch::Ambiguous(matches),
    }
}

#[cfg(test)]
mod tests {
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
}
