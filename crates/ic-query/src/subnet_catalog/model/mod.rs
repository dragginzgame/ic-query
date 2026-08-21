mod classification;
mod policy;
mod types;
mod validation;

pub use classification::{ClassificationSource, GeographicScope, SubnetKind, SubnetSpecialization};
pub use policy::{CLASSIFICATION_SCHEMA_VERSION, RESOLVER_SCHEMA_VERSION};
#[cfg(feature = "subnet-catalog-host")]
pub use types::UncertifiedCatalogCollection;
pub use types::{
    CatalogAssurance, CatalogValidationContext, CertifiedRegistryCatalogEvidence, RawSubnetCatalog,
    RoutingRange, SubnetCatalogProvenance, SubnetCatalogRegistryRecordEvidence,
    SubnetCatalogRegistryRecordKind, SubnetCatalogRegistryRecordSubject,
    SubnetCatalogRegistryValueEncoding, SubnetCatalogRoutingSource, SubnetInfo,
    ValidatedSubnetCatalog,
};
#[cfg(feature = "certified-subnet-catalog-host")]
pub use validation::canonicalize_subnet_catalog_content;
#[cfg(feature = "subnet-catalog-host")]
pub(in crate::subnet_catalog) use validation::catalog_agreement_digest;
