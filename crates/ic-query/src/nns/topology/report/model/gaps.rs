use super::NnsTopologyAssessmentStatus;
use serde::{Deserialize, Serialize};

///
/// NnsTopologyGapsReport
///
/// NNS topology report listing unresolved registry relations.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyGapsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: NnsTopologyAssessmentStatus,
    pub gap_count: usize,
    pub gaps: Vec<NnsTopologyGapRow>,
}

///
/// NnsTopologyGapRow
///
/// One unresolved subject found while joining topology inputs.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyGapRow {
    pub subject_kind: NnsTopologyGapSubjectKind,
    pub subject: String,
    pub missing_relation: NnsTopologyGapRelationKind,
    pub referenced_id: String,
}

///
/// NnsTopologyGapSubjectKind
///
/// Registry subject whose referenced topology relation is missing.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsTopologyGapSubjectKind {
    /// A Registry node record or Subnet membership node.
    Node,
    /// A Registry node-operator record.
    NodeOperator,
}

impl NnsTopologyGapSubjectKind {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::NodeOperator => "node_operator",
        }
    }
}

///
/// NnsTopologyGapRelationKind
///
/// Required Registry relation missing from a component-topology join.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsTopologyGapRelationKind {
    /// The referenced node provider is absent.
    NodeProvider,
    /// The referenced node operator is absent.
    NodeOperator,
    /// The referenced data center is absent.
    DataCenter,
}

impl NnsTopologyGapRelationKind {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NodeProvider => "node_provider",
            Self::NodeOperator => "node_operator",
            Self::DataCenter => "data_center",
        }
    }
}

#[cfg(test)]
mod tests {
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
}
