use std::collections::{BTreeMap, BTreeSet};

pub(super) fn insert_provider_data_center(
    provider: &str,
    data_center_id: &str,
    data_center_regions: &BTreeMap<String, String>,
    data_center_ids: &mut BTreeMap<String, BTreeSet<String>>,
    region_ids: &mut BTreeMap<String, BTreeSet<String>>,
) {
    data_center_ids
        .entry(provider.to_string())
        .or_default()
        .insert(data_center_id.to_string());
    if let Some(region) = data_center_regions.get(data_center_id) {
        region_ids
            .entry(provider.to_string())
            .or_default()
            .insert(region.clone());
    }
}
