use serde::Serialize;
use std::io::{self, Write};

pub fn write_pretty_json<T, E>(value: &T) -> Result<(), E>
where
    T: Serialize,
    E: From<io::Error> + From<serde_json::Error>,
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_json_to::<T, E>(&mut handle, value)?;
    writeln!(handle)?;
    Ok(())
}

fn write_json_to<T: Serialize, E: From<io::Error> + From<serde_json::Error>>(
    writer: impl Write,
    value: &T,
) -> Result<(), E> {
    serde_json::to_writer_pretty(writer, value).map_err(|error| {
        if let Some(kind) = error.io_error_kind() {
            E::from(io::Error::new(kind, error))
        } else {
            E::from(error)
        }
    })
}

pub fn write_help(text: &str) -> io::Result<()> {
    io::stdout().lock().write_all(text.as_bytes())
}

pub fn write_text<E>(text: &str) -> Result<(), E>
where
    E: From<io::Error>,
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{text}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
        let error: CacheCommandError = write_json_to(ClosedWriter, &vec![1, 2])
            .expect_err("closed output is a writer failure");
        assert!(
            matches!(error, CacheCommandError::Io(error) if error.kind() == io::ErrorKind::BrokenPipe)
        );
    }
}
