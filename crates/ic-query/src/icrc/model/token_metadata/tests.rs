use super::*;

#[test]
fn metadata_value_kind_labels_round_trip() {
    for (kind, label) in [
        (IcrcMetadataValueKind::Nat, "nat"),
        (IcrcMetadataValueKind::Int, "int"),
        (IcrcMetadataValueKind::Text, "text"),
        (IcrcMetadataValueKind::Blob, "blob"),
    ] {
        assert_eq!(
            serde_json::to_string(&kind).unwrap(),
            format!("\"{label}\"")
        );
        assert_eq!(
            serde_json::from_str::<IcrcMetadataValueKind>(&format!("\"{label}\"")).unwrap(),
            kind
        );
        assert_eq!(kind.as_str(), label);
    }
}
