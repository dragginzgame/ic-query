mod compact;
mod ranges;
mod verbose;

pub use compact::subnet_catalog_list_report_text;
pub use verbose::subnet_catalog_list_report_verbose_text;

fn subnet_cells(subnet: &crate::subnet_catalog::SubnetCatalogSubnetRow) -> [String; 6] {
    [
        subnet.classification.subnet_kind.as_str().to_string(),
        subnet
            .classification
            .subnet_specialization
            .as_str()
            .to_string(),
        subnet.classification.geographic_scope.as_str().to_string(),
        subnet
            .node_count
            .map_or_else(|| "unknown".to_string(), |count| count.to_string()),
        crate::text_value::yes_no(subnet.charges_apply_by_default).to_string(),
        subnet.range_count.to_string(),
    ]
}
