use crate::subnet_catalog::{
    ClassificationSource, GeographicScope, SubnetCatalog, SubnetSpecialization,
};
use std::collections::BTreeMap;

const FIDUCIARY_SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
const EUROPEAN_SUBNET: &str = "bkfrj-6k62g-dycql-7h53p-atvkj-zg4to-gaogh-netha-ptybj-ntsgw-rqe";

pub(super) fn apply_mainnet_annotations(catalog: &mut SubnetCatalog) {
    let annotations = mainnet_annotations();
    for subnet in &mut catalog.subnets {
        let Some(annotation) = annotations.get(subnet.subnet_principal.as_str()) else {
            continue;
        };
        subnet.subnet_specialization = annotation.specialization;
        subnet.subnet_specialization_source = ClassificationSource::Curated;
        subnet.geographic_scope = annotation.geographic_scope;
        subnet.geographic_scope_source = ClassificationSource::Curated;
        subnet.subnet_label.clone_from(&annotation.label);
        subnet.subnet_label_source = ClassificationSource::Curated;
    }
}

fn mainnet_annotations() -> BTreeMap<&'static str, MainnetAnnotation> {
    BTreeMap::from([
        (
            FIDUCIARY_SUBNET,
            MainnetAnnotation {
                specialization: SubnetSpecialization::Fiduciary,
                geographic_scope: GeographicScope::Global,
                label: "fiduciary".to_string(),
            },
        ),
        (
            EUROPEAN_SUBNET,
            MainnetAnnotation {
                specialization: SubnetSpecialization::European,
                geographic_scope: GeographicScope::Europe,
                label: "european".to_string(),
            },
        ),
    ])
}

///
/// MainnetAnnotation
///
#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetAnnotation {
    specialization: SubnetSpecialization,
    geographic_scope: GeographicScope,
    label: String,
}
