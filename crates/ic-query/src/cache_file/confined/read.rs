//! Module: cache_file::confined::read
//!
//! Responsibility: confined managed-file existence checks and bounded reads.
//! Does not own: path confinement, cache schemas, or atomic publication.
//! Boundary: opens only validated regular files beneath a capability root.

use super::{CacheFileError, ConfinedCacheRoot, ConfinedManagedPath};
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};

///
/// BoundedManagedFileReadError
///
/// Mechanical failure while reading one confined managed file under an explicit byte ceiling.
///

#[derive(Debug)]
pub enum BoundedManagedFileReadError {
    /// Opening or validating the managed path failed.
    Operation(CacheFileError),
    /// Reading file metadata or bytes failed after the path was opened.
    Read {
        /// Managed file being read.
        path: PathBuf,
        /// Underlying filesystem failure.
        source: io::Error,
    },
    /// The file exceeded the caller-selected byte ceiling.
    LimitExceeded {
        /// Managed file that exceeded its ceiling.
        path: PathBuf,
        /// Observed metadata or streamed byte length.
        actual: u64,
        /// Caller-selected maximum byte length.
        maximum: u64,
    },
    /// A platform byte count could not be represented safely.
    Accounting {
        /// Managed file whose byte count could not be represented.
        path: PathBuf,
    },
}

/// Return whether a confined regular managed file exists.
pub fn managed_file_exists(cache_root: &Path, target_path: &Path) -> Result<bool, CacheFileError> {
    let Some(root) = ConfinedCacheRoot::open(cache_root, false)? else {
        return Ok(false);
    };
    let Some(target) = root.resolve_parent(target_path, false)? else {
        return Ok(false);
    };
    Ok(target.open_regular_file()?.is_some())
}

/// Open a confined regular managed file without following symbolic links.
pub fn open_managed_file(
    cache_root: &Path,
    target_path: &Path,
) -> Result<Option<cap_std::fs::File>, CacheFileError> {
    let Some(root) = ConfinedCacheRoot::open(cache_root, false)? else {
        return Ok(None);
    };
    let Some(target) = root.resolve_parent(target_path, false)? else {
        return Ok(None);
    };
    target.open_regular_file()
}

/// Read a confined regular managed file under an explicit byte ceiling.
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "subnet-catalog-host",
    feature = "dashboard-host",
    feature = "nns-topology-host",
    feature = "icrc-host",
    feature = "sns-host",
    test
))]
pub fn read_bounded_managed_file(
    cache_root: &Path,
    target_path: &Path,
    maximum: u64,
) -> Result<Option<Vec<u8>>, BoundedManagedFileReadError> {
    let Some(file) = open_managed_file(cache_root, target_path)
        .map_err(BoundedManagedFileReadError::Operation)?
    else {
        return Ok(None);
    };
    read_opened_file_bounded(file, target_path, maximum).map(Some)
}

impl ConfinedManagedPath {
    pub(in crate::cache_file) fn read_bounded(
        &self,
        maximum: u64,
    ) -> Result<Option<Vec<u8>>, BoundedManagedFileReadError> {
        let Some(file) = self
            .open_regular_file()
            .map_err(BoundedManagedFileReadError::Operation)?
        else {
            return Ok(None);
        };
        read_opened_file_bounded(file, &self.display_path, maximum).map(Some)
    }
}

fn read_opened_file_bounded(
    file: cap_std::fs::File,
    target_path: &Path,
    maximum: u64,
) -> Result<Vec<u8>, BoundedManagedFileReadError> {
    let metadata_length = file
        .metadata()
        .map_err(|source| BoundedManagedFileReadError::Read {
            path: target_path.to_path_buf(),
            source,
        })?
        .len();
    read_bounded_stream(file, metadata_length, target_path, maximum)
}

fn read_bounded_stream(
    mut reader: impl Read,
    metadata_length: u64,
    target_path: &Path,
    maximum: u64,
) -> Result<Vec<u8>, BoundedManagedFileReadError> {
    if metadata_length > maximum {
        return Err(BoundedManagedFileReadError::LimitExceeded {
            path: target_path.to_path_buf(),
            actual: metadata_length,
            maximum,
        });
    }
    let capacity =
        usize::try_from(metadata_length).map_err(|_| BoundedManagedFileReadError::Accounting {
            path: target_path.to_path_buf(),
        })?;
    let mut data = Vec::with_capacity(capacity);
    Read::by_ref(&mut reader)
        .take(maximum.saturating_add(1))
        .read_to_end(&mut data)
        .map_err(|source| BoundedManagedFileReadError::Read {
            path: target_path.to_path_buf(),
            source,
        })?;
    let actual =
        u64::try_from(data.len()).map_err(|_| BoundedManagedFileReadError::Accounting {
            path: target_path.to_path_buf(),
        })?;
    if actual > maximum {
        return Err(BoundedManagedFileReadError::LimitExceeded {
            path: target_path.to_path_buf(),
            actual,
            maximum,
        });
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_stream_rejects_growth_after_metadata_admission() {
        let mut reader = io::Cursor::new(b"123456789-and-more");
        let error = read_bounded_stream(&mut reader, 4, Path::new("catalog.json"), 8)
            .expect_err("growth exceeds the admitted ceiling");
        assert!(matches!(
            error,
            BoundedManagedFileReadError::LimitExceeded {
                actual: 9,
                maximum: 8,
                ..
            }
        ));
        assert_eq!(reader.position(), 9, "read stops at maximum plus one");
    }
}
