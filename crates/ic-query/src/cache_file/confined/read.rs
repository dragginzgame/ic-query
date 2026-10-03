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

///
/// ManagedReadBudget
///
/// Aggregate byte allowance shared by every file read in one local inspection.
///

#[derive(Debug)]
pub struct ManagedReadBudget {
    maximum: u64,
    remaining: u64,
}

impl ManagedReadBudget {
    /// Set the total byte allowance for one inspection operation.
    #[cfg(any(feature = "sns-host", test))]
    pub const fn new(maximum: u64) -> Self {
        Self {
            maximum,
            remaining: maximum,
        }
    }
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

/// Read under a per-file ceiling and an optional shared inspection allowance.
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
    budget: Option<&mut ManagedReadBudget>,
) -> Result<Option<Vec<u8>>, BoundedManagedFileReadError> {
    let Some(file) = open_managed_file(cache_root, target_path)
        .map_err(BoundedManagedFileReadError::Operation)?
    else {
        return Ok(None);
    };
    read_opened_file_bounded(file, target_path, maximum, budget).map(Some)
}

fn read_budgeted_stream(
    reader: impl Read,
    metadata_length: u64,
    target_path: &Path,
    maximum: u64,
    budget: &mut ManagedReadBudget,
) -> Result<Vec<u8>, BoundedManagedFileReadError> {
    if metadata_length > maximum {
        return read_bounded_stream(reader, metadata_length, target_path, maximum);
    }
    let aggregate_maximum = budget.maximum;
    let exhausted = || {
        BoundedManagedFileReadError::Operation(CacheFileError::ScanLimitExceeded {
            path: target_path.to_path_buf(),
            resource: "inspection bytes",
            maximum: aggregate_maximum,
        })
    };
    if metadata_length > budget.remaining {
        return Err(exhausted());
    }
    let ceiling = maximum.min(budget.remaining);
    let result = read_bounded_stream(reader, metadata_length, target_path, ceiling);
    match &result {
        Ok(data) => budget.remaining -= data.len() as u64,
        Err(BoundedManagedFileReadError::LimitExceeded { actual, .. })
            if *actual > ceiling && ceiling == budget.remaining =>
        {
            budget.remaining = 0;
            return Err(exhausted());
        }
        Err(BoundedManagedFileReadError::Read { .. }) => {
            // Partial IO failures still consume work; conservatively charge the ceiling.
            budget.remaining = budget.remaining.saturating_sub(ceiling.saturating_add(1));
        }
        Err(BoundedManagedFileReadError::LimitExceeded { .. }) => budget.remaining -= ceiling + 1,
        Err(_) => {}
    }
    result
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
        read_opened_file_bounded(file, &self.display_path, maximum, None).map(Some)
    }
}

fn read_opened_file_bounded(
    file: cap_std::fs::File,
    target_path: &Path,
    maximum: u64,
    budget: Option<&mut ManagedReadBudget>,
) -> Result<Vec<u8>, BoundedManagedFileReadError> {
    let metadata_length = file
        .metadata()
        .map_err(|source| BoundedManagedFileReadError::Read {
            path: target_path.to_path_buf(),
            source,
        })?
        .len();
    match budget {
        Some(budget) => read_budgeted_stream(file, metadata_length, target_path, maximum, budget),
        None => read_bounded_stream(file, metadata_length, target_path, maximum),
    }
}

pub fn read_bounded_stream(
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

    struct FailsAfterBytes(bool);
    impl Read for FailsAfterBytes {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.0 {
                return Err(io::Error::other("fixture read failure"));
            }
            self.0 = true;
            buffer[0] = b'x';
            Ok(1)
        }
    }

    #[test]
    fn aggregate_reads_charge_actual_bytes_and_stop_before_oversized_allocation() {
        let path = Path::new("snapshot.json");
        let mut budget = ManagedReadBudget::new(10);
        let first =
            read_budgeted_stream(io::Cursor::new(b"1234"), 4, path, 100, &mut budget).unwrap();
        assert_eq!(first, b"1234");
        let mut second = io::Cursor::new(b"1234567");
        assert!(matches!(
            read_budgeted_stream(&mut second, 7, path, 100, &mut budget),
            Err(BoundedManagedFileReadError::Operation(
                CacheFileError::ScanLimitExceeded {
                    resource: "inspection bytes",
                    maximum: 10,
                    ..
                }
            ))
        ));
        assert_eq!(second.position(), 0);
        assert_eq!(
            read_budgeted_stream(io::Cursor::new(b"123456"), 6, path, 100, &mut budget).unwrap(),
            b"123456"
        );
        assert!(read_budgeted_stream(io::Cursor::new(b"x"), 1, path, 100, &mut budget).is_err());
    }

    #[test]
    fn aggregate_reads_bound_growth_and_charge_partial_io_failures() {
        let path = Path::new("snapshot.json");
        let mut budget = ManagedReadBudget::new(8);
        let mut reader = io::Cursor::new(b"123456789-and-more");
        assert!(matches!(
            read_budgeted_stream(&mut reader, 4, path, 100, &mut budget),
            Err(BoundedManagedFileReadError::Operation(
                CacheFileError::ScanLimitExceeded { .. }
            ))
        ));
        assert_eq!(reader.position(), 9);
        assert_eq!(budget.remaining, 0);

        let mut budget = ManagedReadBudget::new(20);
        let mut failed = io::Cursor::new(b"1234");
        // Growth beyond a per-file limit is still charged to the aggregate.
        assert!(matches!(
            read_budgeted_stream(&mut failed, 3, path, 3, &mut budget),
            Err(BoundedManagedFileReadError::LimitExceeded { .. })
        ));
        assert_eq!(budget.remaining, 16);

        assert!(matches!(
            read_budgeted_stream(FailsAfterBytes(false), 1, path, 3, &mut budget),
            Err(BoundedManagedFileReadError::Read { .. })
        ));
        assert_eq!(budget.remaining, 12);
    }

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
