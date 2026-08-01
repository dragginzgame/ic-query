//! Module: cli::clap
//!
//! Responsibility: small clap helper wrappers shared by command parsers.
//! Does not own: command-family specs, report requests, or runtime dispatch.
//! Boundary: normalizes typed values, parse errors, and help rendering.

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::ffi::OsString;

pub fn parse_matches<I>(command: Command, args: I) -> Result<ArgMatches, clap::Error>
where
    I: IntoIterator<Item = OsString>,
{
    let name = command.get_name().to_string();
    command.try_get_matches_from(std::iter::once(OsString::from(name)).chain(args))
}

#[cfg(test)]
pub fn parse_matches_or_usage<I>(command: Command, args: I) -> Result<ArgMatches, String>
where
    I: IntoIterator<Item = OsString>,
{
    let mut help_command = command.clone();
    parse_matches(command, args).map_err(|error| format!("{error}\n{}", help_command.render_help()))
}

pub fn value_arg(id: &'static str) -> Arg {
    Arg::new(id).num_args(1)
}

pub fn flag_arg(id: &'static str) -> Arg {
    Arg::new(id).action(ArgAction::SetTrue)
}

pub fn string_option(matches: &ArgMatches, id: &str) -> Option<String> {
    matches.get_one::<String>(id).cloned()
}

/// Returns a required string argument that clap has already validated.
///
/// # Panics
///
/// Panics when `id` is not present in `matches`. Callers should only use this
/// for arguments declared as required in the same clap command definition.
pub fn required_string(matches: &ArgMatches, id: &str) -> String {
    string_option(matches, id).unwrap_or_else(|| panic!("clap requires {id}"))
}

pub fn typed_option<T>(matches: &ArgMatches, id: &str) -> Option<T>
where
    T: Clone + Send + Sync + 'static,
{
    matches.get_one::<T>(id).cloned()
}

/// Returns a required typed argument that clap has already validated.
///
/// # Panics
///
/// Panics when `id` is not present in `matches`. Callers should only use this
/// for arguments declared as required in the same clap command definition.
pub fn required_typed<T>(matches: &ArgMatches, id: &str) -> T
where
    T: Clone + Send + Sync + 'static,
{
    typed_option(matches, id).unwrap_or_else(|| panic!("clap requires {id}"))
}

#[cfg(test)]
pub fn render_help(mut command: Command) -> String {
    command.render_help().to_string()
}
