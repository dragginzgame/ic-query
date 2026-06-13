use serde::Serialize;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

// Write a pretty JSON payload to a requested file or stdout.
pub fn write_pretty_json<T, E>(out: Option<&PathBuf>, value: &T) -> Result<(), E>
where
    T: Serialize,
    E: From<io::Error> + From<serde_json::Error>,
{
    if let Some(path) = out {
        ensure_parent_dir::<E>(path)?;
        let data = serde_json::to_vec_pretty(value)?;
        fs::write(path, data)?;
        return Ok(());
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value)?;
    writeln!(handle)?;
    Ok(())
}

// Write a plain text payload to a requested file or stdout.
pub fn write_text<E>(out: Option<&PathBuf>, text: &str) -> Result<(), E>
where
    E: From<io::Error>,
{
    if let Some(path) = out {
        ensure_parent_dir::<E>(path)?;
        fs::write(path, text)?;
    } else {
        println!("{text}");
    }
    Ok(())
}

fn ensure_parent_dir<E>(path: &Path) -> Result<(), E>
where
    E: From<io::Error>,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
