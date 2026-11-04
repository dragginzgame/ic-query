use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Row(&'static str);

#[test]
fn resolves_normalized_text_exactly_and_principals_by_prefix() {
    let text_rows = [Row("dc1")];
    assert_eq!(
        resolve_nns_inventory_row(
            &text_rows,
            |row| row.0,
            " DC1 ",
            NnsInventoryInputKind::Text,
            "id"
        ),
        Ok((0, "id".to_string()))
    );

    assert_eq!(
        resolve_nns_inventory_row(
            &text_rows,
            |row| row.0,
            " DC ",
            NnsInventoryInputKind::Text,
            "id"
        ),
        Ok((0, "id_prefix".to_string()))
    );

    let principal_rows = [Row("ryjl3-tyaaa-aaaaa-aaaba-cai")];
    assert_eq!(
        resolve_nns_inventory_row(
            &principal_rows,
            |row| row.0,
            "ryjl",
            NnsInventoryInputKind::Principal,
            "principal",
        ),
        Ok((0, "principal_prefix".to_string(),))
    );
}

#[test]
fn preserves_not_found_input_and_ambiguous_matches() {
    let rows = [Row("dc1"), Row("dc2")];
    assert_eq!(
        resolve_nns_inventory_row(&rows, |row| row.0, " ", NnsInventoryInputKind::Text, "id"),
        Err(NnsInventoryResolveError::NotFound {
            input: " ".to_string(),
        })
    );
    assert_eq!(
        resolve_nns_inventory_row(&rows, |row| row.0, "DC", NnsInventoryInputKind::Text, "id"),
        Err(NnsInventoryResolveError::Ambiguous {
            prefix: "dc".to_string(),
            matches: vec!["dc1".to_string(), "dc2".to_string()],
        })
    );
}
