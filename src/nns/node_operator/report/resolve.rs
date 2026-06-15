use super::{NnsNodeOperatorHostError, NnsNodeOperatorListReport, NnsNodeOperatorRow};
use crate::subnet_catalog::canonical_principal_text;

pub(super) fn resolve_node_operator(
    report: &NnsNodeOperatorListReport,
    input: &str,
) -> Result<(NnsNodeOperatorRow, String), NnsNodeOperatorHostError> {
    if let Ok(principal) = canonical_principal_text(input)
        && let Some(operator) = report
            .node_operators
            .iter()
            .find(|operator| operator.node_operator_principal == principal)
    {
        return Ok((operator.clone(), "node_operator_principal".to_string()));
    }

    let prefix = input.trim().to_ascii_lowercase();
    if prefix.is_empty() {
        return Err(NnsNodeOperatorHostError::NodeOperatorNotFound {
            input: input.to_string(),
        });
    }
    let matches = report
        .node_operators
        .iter()
        .filter(|operator| operator.node_operator_principal.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [operator] => Ok((
            operator.clone(),
            "node_operator_principal_prefix".to_string(),
        )),
        [] => Err(NnsNodeOperatorHostError::NodeOperatorNotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsNodeOperatorHostError::AmbiguousNodeOperatorPrefix {
            prefix,
            matches: matches
                .into_iter()
                .map(|operator| operator.node_operator_principal)
                .collect(),
        }),
    }
}
