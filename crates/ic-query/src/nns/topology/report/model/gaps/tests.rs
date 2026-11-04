use super::*;

#[test]
fn topology_gap_labels_round_trip() {
    assert_labels(&[
        (NnsTopologyGapSubjectKind::Node, "node"),
        (NnsTopologyGapSubjectKind::NodeOperator, "node_operator"),
    ]);
    assert_labels(&[
        (NnsTopologyGapRelationKind::NodeProvider, "node_provider"),
        (NnsTopologyGapRelationKind::NodeOperator, "node_operator"),
        (NnsTopologyGapRelationKind::DataCenter, "data_center"),
    ]);
}

fn assert_labels<T>(cases: &[(T, &str)])
where
    T: Copy + std::fmt::Debug + Eq + Serialize + serde::de::DeserializeOwned,
{
    for &(value, label) in cases {
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            format!("\"{label}\"")
        );
        assert_eq!(
            serde_json::from_str::<T>(&format!("\"{label}\"")).unwrap(),
            value
        );
    }
}
