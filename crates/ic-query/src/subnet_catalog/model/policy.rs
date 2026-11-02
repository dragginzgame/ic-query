//! Module: subnet_catalog::model::policy
//!
//! Responsibility: own reproducible Subnet classification and resolver policy identity.
//! Does not own: Registry transport, cache loading, or report rendering.
//! Boundary: applies and validates the annotations committed by the policy digest.

#[cfg(feature = "subnet-catalog-host")]
use super::classification::REGISTRY_SUBNET_KINDS;
#[cfg(feature = "subnet-catalog-host")]
use super::{
    ClassificationSource, GeographicScope, SubnetClassification, SubnetInfo, SubnetSpecialization,
};
#[cfg(feature = "subnet-catalog-host")]
use ic_host_artifacts::artifact::Sha256Digest;
#[cfg(feature = "subnet-catalog-host")]
use std::fmt::Write;

/// Current classification policy schema.
pub const CLASSIFICATION_SCHEMA_VERSION: u32 = 1;
/// Current inclusive principal-byte resolver schema.
pub const RESOLVER_SCHEMA_VERSION: u32 = 1;

#[cfg(feature = "subnet-catalog-host")]
pub(super) const RESOLVER_BACKEND: &str = "local-nns-subnet-catalog";

#[cfg(feature = "subnet-catalog-host")]
struct CuratedClassification {
    principal: &'static str,
    specialization: SubnetSpecialization,
    geographic_scope: GeographicScope,
}

#[cfg(feature = "subnet-catalog-host")]
const CURATED_CLASSIFICATIONS: &[CuratedClassification] = &[
    CuratedClassification {
        principal: "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae",
        specialization: SubnetSpecialization::Fiduciary,
        geographic_scope: GeographicScope::Global,
    },
    CuratedClassification {
        principal: "bkfrj-6k62g-dycql-7h53p-atvkj-zg4to-gaogh-netha-ptybj-ntsgw-rqe",
        specialization: SubnetSpecialization::European,
        geographic_scope: GeographicScope::Europe,
    },
];

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
pub(super) fn classification_policy_digest() -> String {
    let kinds = REGISTRY_SUBNET_KINDS
        .iter()
        .map(|(raw, kind)| format!("{raw}={}", kind.as_str()))
        .collect::<Vec<_>>()
        .join(",");
    let charges = super::SubnetKind::ALL
        .into_iter()
        .filter(|kind| kind.charges_apply_by_default())
        .map(super::SubnetKind::as_str)
        .collect::<Vec<_>>()
        .join("|");
    let mut policy = format!(
        "ic-query/subnet-catalog/classification/v1;subnet_type:{kinds},other={};charges:{charges};defaults:specialization={},geographic_scope={},label=subnet_kind,sources={}|{};",
        super::SubnetKind::Unknown.as_str(),
        SubnetSpecialization::None.as_str(),
        GeographicScope::Global.as_str(),
        ClassificationSource::Registry.as_str(),
        ClassificationSource::Computed.as_str()
    );
    for rule in CURATED_CLASSIFICATIONS {
        let label = rule.specialization.as_str();
        write!(
            policy,
            "{label}:{}={label}|{}|{label};",
            rule.principal,
            rule.geographic_scope.as_str()
        )
        .expect("writing to String cannot fail");
    }
    policy.push_str("subnet_order=canonical_principal_text;routing_order=principal_bytes(start,end),subnet_principal");
    Sha256Digest::compute(policy.as_bytes()).to_string()
}

#[cfg(feature = "subnet-catalog-host")]
pub(super) fn apply_mainnet_classification_policy(subnets: &mut [SubnetInfo]) {
    for subnet in subnets {
        subnet.classification = mainnet_classification(subnet);
    }
}

#[cfg(feature = "subnet-catalog-host")]
pub(super) fn mainnet_classification(subnet: &SubnetInfo) -> SubnetClassification {
    let (specialization, geographic_scope, label, source) = CURATED_CLASSIFICATIONS
        .iter()
        .find(|rule| rule.principal == subnet.subnet_principal)
        .map_or(
            (
                SubnetSpecialization::None,
                GeographicScope::Global,
                subnet.classification.subnet_kind.as_str(),
                ClassificationSource::Computed,
            ),
            |rule| {
                (
                    rule.specialization,
                    rule.geographic_scope,
                    rule.specialization.as_str(),
                    ClassificationSource::Curated,
                )
            },
        );
    SubnetClassification {
        subnet_specialization: specialization,
        geographic_scope,
        subnet_label: label.to_string(),
        subnet_specialization_source: source,
        geographic_scope_source: source,
        subnet_label_source: source,
        registry_subnet_type: subnet.classification.registry_subnet_type,
        subnet_kind: subnet.classification.subnet_kind,
        subnet_kind_source: subnet.classification.subnet_kind_source,
    }
}

#[cfg(all(test, feature = "subnet-catalog-host"))]
mod tests {
    #[test]
    fn policy_table_preserves_existing_fingerprint() {
        assert_eq!(
            super::classification_policy_digest(),
            "a3a544eb8c5a54a93b06583a16e3952598cee5ec7d9629e2678c5ed85bd58a2c"
        );
    }
}
