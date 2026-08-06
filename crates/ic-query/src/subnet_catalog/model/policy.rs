//! Module: subnet_catalog::model::policy
//!
//! Responsibility: own reproducible Subnet classification and resolver policy identity.
//! Does not own: Registry transport, cache loading, or report rendering.
//! Boundary: applies and validates the annotations committed by the policy digest.

#[cfg(feature = "subnet-catalog-host")]
use super::{ClassificationSource, GeographicScope, SubnetInfo, SubnetSpecialization};
#[cfg(feature = "subnet-catalog-host")]
use crate::hex::hex_bytes;
#[cfg(feature = "subnet-catalog-host")]
use sha2::{Digest, Sha256};

/// Current classification policy schema.
pub const CLASSIFICATION_SCHEMA_VERSION: u32 = 1;
/// Current inclusive principal-byte resolver schema.
pub const RESOLVER_SCHEMA_VERSION: u32 = 1;

#[cfg(feature = "subnet-catalog-host")]
pub(super) const RESOLVER_BACKEND: &str = "local-nns-subnet-catalog";

#[cfg(feature = "subnet-catalog-host")]
const FIDUCIARY_SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
#[cfg(feature = "subnet-catalog-host")]
const EUROPEAN_SUBNET: &str = "bkfrj-6k62g-dycql-7h53p-atvkj-zg4to-gaogh-netha-ptybj-ntsgw-rqe";

#[cfg(feature = "subnet-catalog-host")]
const CLASSIFICATION_POLICY: &str = concat!(
    "ic-query/subnet-catalog/classification/v1;",
    "subnet_type:0=unknown,1=application,2=system,4=application,5=cloud_engine,other=unknown;",
    "charges:application|cloud_engine;",
    "defaults:specialization=none,geographic_scope=global,label=subnet_kind,sources=registry|computed;",
    "fiduciary:pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae=",
    "fiduciary|global|fiduciary;",
    "european:bkfrj-6k62g-dycql-7h53p-atvkj-zg4to-gaogh-netha-ptybj-ntsgw-rqe=",
    "european|europe|european;",
    "subnet_order=canonical_principal_text;",
    "routing_order=principal_bytes(start,end),subnet_principal",
);

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
pub(super) fn classification_policy_digest() -> String {
    hex_bytes(&Sha256::digest(CLASSIFICATION_POLICY.as_bytes()))
}

#[cfg(feature = "subnet-catalog-host")]
pub(super) fn apply_mainnet_classification_policy(subnets: &mut [SubnetInfo]) {
    for subnet in subnets {
        apply_default_classification(subnet);
        match subnet.subnet_principal.as_str() {
            FIDUCIARY_SUBNET => apply_curated_classification(
                subnet,
                SubnetSpecialization::Fiduciary,
                GeographicScope::Global,
                "fiduciary",
            ),
            EUROPEAN_SUBNET => apply_curated_classification(
                subnet,
                SubnetSpecialization::European,
                GeographicScope::Europe,
                "european",
            ),
            _ => {}
        }
    }
}

#[cfg(feature = "subnet-catalog-host")]
fn apply_default_classification(subnet: &mut SubnetInfo) {
    subnet.subnet_specialization = SubnetSpecialization::None;
    subnet.subnet_specialization_source = ClassificationSource::Computed;
    subnet.geographic_scope = GeographicScope::Global;
    subnet.geographic_scope_source = ClassificationSource::Computed;
    subnet.subnet_label = subnet.subnet_kind.as_str().to_string();
    subnet.subnet_label_source = ClassificationSource::Computed;
}

#[cfg(feature = "subnet-catalog-host")]
fn apply_curated_classification(
    subnet: &mut SubnetInfo,
    specialization: SubnetSpecialization,
    geographic_scope: GeographicScope,
    label: &str,
) {
    subnet.subnet_specialization = specialization;
    subnet.subnet_specialization_source = ClassificationSource::Curated;
    subnet.geographic_scope = geographic_scope;
    subnet.geographic_scope_source = ClassificationSource::Curated;
    subnet.subnet_label = label.to_string();
    subnet.subnet_label_source = ClassificationSource::Curated;
}
