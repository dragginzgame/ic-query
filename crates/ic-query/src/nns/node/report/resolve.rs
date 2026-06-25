use super::{NnsNodeHostError, NnsNodeListReport, NnsNodeRow};
use crate::subnet_catalog::canonical_principal_text;

pub(super) fn resolve_node(
    report: &NnsNodeListReport,
    input: &str,
) -> Result<(NnsNodeRow, String), NnsNodeHostError> {
    if let Ok(principal) = canonical_principal_text(input)
        && let Some(node) = report
            .nodes
            .iter()
            .find(|node| node.node_principal == principal)
    {
        return Ok((node.clone(), "node_principal".to_string()));
    }

    let prefix = input.trim().to_ascii_lowercase();
    if prefix.is_empty() {
        return Err(NnsNodeHostError::NodeNotFound {
            input: input.to_string(),
        });
    }
    let matches = report
        .nodes
        .iter()
        .filter(|node| node.node_principal.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [node] => Ok((node.clone(), "node_principal_prefix".to_string())),
        [] => Err(NnsNodeHostError::NodeNotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsNodeHostError::AmbiguousNodePrefix {
            prefix,
            matches: matches
                .into_iter()
                .map(|node| node.node_principal)
                .collect(),
        }),
    }
}
