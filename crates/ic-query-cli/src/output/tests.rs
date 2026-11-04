use super::*;
use crate::cache::CacheCommandError;

struct ClosedWriter;

impl Write for ClosedWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn json_writer_preserves_io_error_kind() {
    let error: CacheCommandError =
        write_json_to(ClosedWriter, &vec![1, 2]).expect_err("closed output is a writer failure");
    assert!(
        matches!(error, CacheCommandError::Io(error) if error.kind() == io::ErrorKind::BrokenPipe)
    );
}
