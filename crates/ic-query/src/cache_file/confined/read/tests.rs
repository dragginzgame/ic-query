use super::*;

struct InterruptedOnce<R>(bool, R);

impl<R: Read> Read for InterruptedOnce<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if !self.0 {
            self.0 = true;
            return Err(io::ErrorKind::Interrupted.into());
        }
        self.1.read(buffer)
    }
}

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
    let first = read_budgeted_stream(io::Cursor::new(b"1234"), 4, path, 100, &mut budget).unwrap();
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

#[test]
fn metadata_overflow_is_rejected_before_reading() {
    let mut reader = io::Cursor::new(b"fixture");
    assert!(matches!(
        read_bounded_stream(&mut reader, 10, Path::new("snapshot.json"), 8),
        Err(BoundedManagedFileReadError::LimitExceeded {
            actual: 10,
            maximum: 8,
            ..
        })
    ));
    assert_eq!(reader.position(), 0);
}

#[test]
fn stream_reads_accept_exact_zero_and_unbounded_ceilings() {
    let path = Path::new("snapshot.json");
    assert_eq!(read_bounded_stream(io::empty(), 0, path, 0).unwrap(), b"");
    assert_eq!(
        read_bounded_stream(io::Cursor::new(b"1234"), 0, path, 4).unwrap(),
        b"1234"
    );
    assert!(matches!(
        read_bounded_stream(io::Cursor::new(b"x"), 0, path, 0),
        Err(BoundedManagedFileReadError::LimitExceeded {
            actual: 1,
            maximum: 0,
            ..
        })
    ));
    assert_eq!(
        read_bounded_stream(io::Cursor::new(b"1234"), 0, path, u64::MAX).unwrap(),
        b"1234"
    );
    let mut reader = io::Cursor::new(b"x");
    let result = read_bounded_stream(&mut reader, u64::MAX, path, u64::MAX);
    if usize::try_from(u64::MAX).is_ok() {
        assert_eq!(result.unwrap(), b"x");
    } else {
        assert!(matches!(
            result,
            Err(BoundedManagedFileReadError::Accounting { .. })
        ));
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn interrupted_stream_reads_retry_without_losing_bytes() {
    let reader = InterruptedOnce(false, io::Cursor::new(b"fixture"));
    assert_eq!(
        read_bounded_stream(reader, 7, Path::new("snapshot.json"), 7).unwrap(),
        b"fixture"
    );
}
