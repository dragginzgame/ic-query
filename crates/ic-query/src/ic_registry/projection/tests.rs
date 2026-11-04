use super::*;

#[test]
fn registry_subnet_type_conversion_preserves_raw_ic_query_kinds() {
    let cases = [
        (SubnetType::Application as i32, SubnetKind::Application),
        (
            SubnetType::VerifiedApplication as i32,
            SubnetKind::Application,
        ),
        (SubnetType::System as i32, SubnetKind::System),
        (SubnetType::CloudEngine as i32, SubnetKind::CloudEngine),
        (SubnetType::Unspecified as i32, SubnetKind::Unknown),
        (i32::MAX, SubnetKind::Unknown),
    ];

    for (raw_subnet_type, expected) in cases {
        assert_eq!(subnet_kind_from_registry(raw_subnet_type), expected);
    }
}
