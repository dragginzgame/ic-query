use super::{NnsDataCenterHostError, NnsDataCenterListReport, NnsDataCenterRow};

pub(super) fn resolve_data_center(
    report: &NnsDataCenterListReport,
    input: &str,
) -> Result<(NnsDataCenterRow, String), NnsDataCenterHostError> {
    let normalized = input.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(NnsDataCenterHostError::DataCenterNotFound {
            input: input.to_string(),
        });
    }
    if let Some(data_center) = report
        .data_centers
        .iter()
        .find(|data_center| data_center.data_center_id == normalized)
    {
        return Ok((data_center.clone(), "data_center_id".to_string()));
    }
    let matches = report
        .data_centers
        .iter()
        .filter(|data_center| data_center.data_center_id.starts_with(&normalized))
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [data_center] => Ok((data_center.clone(), "data_center_id_prefix".to_string())),
        [] => Err(NnsDataCenterHostError::DataCenterNotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsDataCenterHostError::AmbiguousDataCenterPrefix {
            prefix: normalized,
            matches: matches
                .into_iter()
                .map(|data_center| data_center.data_center_id)
                .collect(),
        }),
    }
}
