use super::validate_canonical;

#[test]
fn strict_evidence_admits_canonical_text_and_distinguishes_invalid_spelling() {
    for value in [
        "aaaaa-aa",
        "2vxsx-fae",
        "AAAAA-AA",
        "2VXSX-FAE",
        "aaaaaaa",
        " aaaaa-aa",
        "invalid",
        "é",
    ] {
        let result = validate_canonical(value, |_| None, Some);
        match value {
            "aaaaa-aa" | "2vxsx-fae" => assert_eq!(result, Ok(())),
            "AAAAA-AA" => assert_eq!(result, Err(Some("aaaaa-aa".to_string()))),
            "2VXSX-FAE" => assert_eq!(result, Err(Some("2vxsx-fae".to_string()))),
            _ => assert_eq!(result, Err(None)),
        }
    }
}
