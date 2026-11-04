mod request;

pub use request::MainnetRegistryFetchRequest;

/// Rows and Registry acquisition evidence passed to the report owner.
#[cfg(feature = "nns-host")]
pub(in crate::ic_registry) type RegistryInventoryRows<Row> = (
    crate::nns::registry::NnsRegistryInventoryProvenance,
    Vec<Row>,
);
