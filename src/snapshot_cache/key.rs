///
/// SnapshotKey
///
/// Stable identity for a complete snapshot independent of display options.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotKey {
    domain: String,
    network: String,
    entity: String,
    collection: String,
    scope: SnapshotScope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SnapshotScope {
    Full,
}

impl SnapshotKey {
    pub fn full(
        domain: impl Into<String>,
        network: impl Into<String>,
        entity: impl Into<String>,
        collection: impl Into<String>,
    ) -> Self {
        Self {
            domain: domain.into(),
            network: network.into(),
            entity: entity.into(),
            collection: collection.into(),
            scope: SnapshotScope::Full,
        }
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn network(&self) -> &str {
        &self.network
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn collection(&self) -> &str {
        &self.collection
    }

    pub const fn scope_file_stem(&self) -> &'static str {
        match self.scope {
            SnapshotScope::Full => "full",
        }
    }
}
