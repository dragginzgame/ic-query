//! Module: subnet_catalog::model::policy
//!
//! Responsibility: own reproducible Subnet classification and resolver policy identity.
//! Does not own: Registry transport, cache loading, or report rendering.
//! Boundary: applies and validates the annotations committed by the policy digest.

#[cfg(feature = "subnet-catalog-host")]
use super::{ClassificationSource, GeographicScope, SubnetInfo, SubnetSpecialization};
#[cfg(feature = "subnet-catalog-host")]
use ic_host_artifacts::artifact::Sha256Digest;

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
    Sha256Digest::compute(CLASSIFICATION_POLICY.as_bytes()).to_string()
}

#[cfg(feature = "subnet-catalog-host")]
pub(super) fn apply_mainnet_classification_policy(subnets: &mut [SubnetInfo]) {
    for subnet in subnets {
        let classification = mainnet_classification(subnet);
        subnet.subnet_specialization = classification.specialization;
        subnet.subnet_specialization_source = classification.source;
        subnet.geographic_scope = classification.geographic_scope;
        subnet.geographic_scope_source = classification.source;
        subnet.subnet_label = classification.label.to_string();
        subnet.subnet_label_source = classification.source;
    }
}

///
/// SubnetClassification
///
/// Derived annotations used to apply and independently check the current policy.
///

#[cfg(feature = "subnet-catalog-host")]
pub(super) struct SubnetClassification {
    specialization: SubnetSpecialization,
    geographic_scope: GeographicScope,
    label: &'static str,
    source: ClassificationSource,
}

#[cfg(feature = "subnet-catalog-host")]
impl SubnetClassification {
    pub(super) fn matches(&self, subnet: &SubnetInfo) -> bool {
        subnet.subnet_specialization == self.specialization
            && subnet.subnet_specialization_source == self.source
            && subnet.geographic_scope == self.geographic_scope
            && subnet.geographic_scope_source == self.source
            && subnet.subnet_label == self.label
            && subnet.subnet_label_source == self.source
    }
}

#[cfg(feature = "subnet-catalog-host")]
pub(super) fn mainnet_classification(subnet: &SubnetInfo) -> SubnetClassification {
    let (specialization, geographic_scope, label, source) = match subnet.subnet_principal.as_str() {
        FIDUCIARY_SUBNET => (
            SubnetSpecialization::Fiduciary,
            GeographicScope::Global,
            "fiduciary",
            ClassificationSource::Curated,
        ),
        EUROPEAN_SUBNET => (
            SubnetSpecialization::European,
            GeographicScope::Europe,
            "european",
            ClassificationSource::Curated,
        ),
        _ => (
            SubnetSpecialization::None,
            GeographicScope::Global,
            subnet.subnet_kind.as_str(),
            ClassificationSource::Computed,
        ),
    };
    SubnetClassification {
        specialization,
        geographic_scope,
        label,
        source,
    }
}
