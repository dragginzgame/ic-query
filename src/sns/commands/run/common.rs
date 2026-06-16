use crate::{
    cli::common::{OutputFormat, current_unix_secs},
    cli::help::collect_args_or_print_help_or_version,
    project::icp_root,
    sns::commands::{SnsCommandError, options::SnsLookupOptions},
    version_text,
};
use std::ffi::OsString;
use std::path::PathBuf;

pub(super) struct SnsLookupCommandParts {
    pub(super) format: OutputFormat,
    pub(super) network: String,
    pub(super) source_endpoint: String,
    pub(super) now_unix_secs: u64,
    pub(super) input: String,
}

pub(super) fn command_unix_secs() -> Result<u64, SnsCommandError> {
    current_unix_secs().map_err(SnsCommandError::Clock)
}

pub(super) fn command_args<I>(args: I, usage: impl FnOnce() -> String) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    collect_args_or_print_help_or_version(args, usage, version_text())
}

pub(super) fn command_icp_root() -> Result<PathBuf, SnsCommandError> {
    icp_root().map_err(|err| SnsCommandError::Usage(err.to_string()))
}

pub(super) fn lookup_command_parts(
    options: SnsLookupOptions,
) -> Result<SnsLookupCommandParts, SnsCommandError> {
    Ok(SnsLookupCommandParts {
        format: options.format,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.input,
    })
}
