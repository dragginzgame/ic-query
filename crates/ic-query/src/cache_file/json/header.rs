//! Top-level JSON cache admission shared by snapshots and refresh-attempt sidecars.

use serde::de::{Error as _, IgnoredAny, MapAccess, Visitor};
use std::{collections::BTreeSet, fmt};

///
/// JsonCacheHeader
///
/// Top-level cache identity and unique field names, admitted without retaining payloads.
///

pub struct JsonCacheHeader {
    pub schema_version: Option<u32>,
    pub network: Option<String>,
    pub keys: BTreeSet<String>,
}

impl<'de> serde::Deserialize<'de> for JsonCacheHeader {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(JsonCacheHeaderVisitor)
    }
}

struct JsonCacheHeaderVisitor;

impl<'de> Visitor<'de> for JsonCacheHeaderVisitor {
    type Value = JsonCacheHeader;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON cache object")
    }

    fn visit_map<Map>(self, mut map: Map) -> Result<Self::Value, Map::Error>
    where
        Map: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        let mut schema_version = None;
        let mut network = None;
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(Map::Error::custom(format!(
                    "duplicate top-level cache field {key:?}"
                )));
            }
            match key.as_str() {
                "schema_version" => schema_version = Some(map.next_value()?),
                "network" => network = map.next_value()?,
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(JsonCacheHeader {
            schema_version,
            network,
            keys,
        })
    }
}
