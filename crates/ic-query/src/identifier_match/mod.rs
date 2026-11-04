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
mod tests;
