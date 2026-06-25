use serde::Serialize;
use std::io::{self, Write};

pub fn write_pretty_json<T, E>(value: &T) -> Result<(), E>
where
    T: Serialize,
    E: From<io::Error> + From<serde_json::Error>,
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value)?;
    writeln!(handle)?;
    Ok(())
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
