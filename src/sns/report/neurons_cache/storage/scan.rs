use crate::sns::report::SnsHostError;
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::super::{
    SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SnsNeuronsCacheHeader, paths::sns_network_cache_dir,
};

pub(super) fn collect_sns_neurons_cache_paths(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError> {
    let root = sns_network_cache_dir(icp_root, network);
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(SnsHostError::ReadCache { path: root, source });
        }
    };
    let mut cache_paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| SnsHostError::ReadCache {
            path: root.clone(),
            source,
        })?;
        let path = entry.path().join("neurons").join("full.json");
        if path.is_file() {
            cache_paths.push(path);
        }
    }
    cache_paths.sort();
    Ok(cache_paths)
}

pub(super) fn read_sns_neurons_cache_header(
    path: &Path,
    network: &str,
) -> Result<SnsNeuronsCacheHeader, SnsHostError> {
    let data = fs::read(path).map_err(|source| SnsHostError::ReadCache {
        path: path.to_path_buf(),
        source,
    })?;
    let header: SnsNeuronsCacheHeader =
        serde_json::from_slice(&data).map_err(|source| SnsHostError::ParseCache {
            path: path.to_path_buf(),
            source,
        })?;
    validate_sns_neurons_cache_header(header, network)
}

fn validate_sns_neurons_cache_header(
    header: SnsNeuronsCacheHeader,
    network: &str,
) -> Result<SnsNeuronsCacheHeader, SnsHostError> {
    if header.schema_version != SNS_NEURONS_CACHE_SCHEMA_VERSION {
        return Err(SnsHostError::UnsupportedCacheSchemaVersion {
            version: header.schema_version,
            expected: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        });
    }
    if header.network != network {
        return Err(SnsHostError::CacheNetworkMismatch {
            requested: network.to_string(),
            actual: header.network,
        });
    }
    Ok(header)
}
