//! Module: cli::clap
//!
//! Responsibility: small clap helper wrappers shared by command parsers.
//! Does not own: command-family specs, report requests, or runtime dispatch.
//! Boundary: normalizes passthrough subcommands, required values, and help rendering.

use clap::{Arg, ArgAction, ArgMatches, Command, error::ErrorKind};
use std::ffi::OsString;

const PASSTHROUGH_ARGS: &str = "args";

///
/// OptionalSubcommand
///
/// Parsed result for commands that accept either a subcommand or passthrough args.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionalSubcommand {
    Matched { name: String, args: Vec<OsString> },
    Passthrough(Vec<OsString>),
}

pub fn parse_matches<I>(command: Command, args: I) -> Result<ArgMatches, clap::Error>
where
    I: IntoIterator<Item = OsString>,
{
    let name = command.get_name().to_string();
    command.try_get_matches_from(std::iter::once(OsString::from(name)).chain(args))
}

pub fn parse_matches_or_usage<I>(
    command: Command,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<ArgMatches, String>
where
    I: IntoIterator<Item = OsString>,
{
    parse_matches(command, args).map_err(|_| usage())
}

pub fn passthrough_subcommand(command: Command) -> Command {
    command.arg(
        Arg::new(PASSTHROUGH_ARGS)
            .num_args(0..)
            .allow_hyphen_values(true)
            .trailing_var_arg(true)
            .value_parser(clap::value_parser!(OsString)),
    )
}

pub fn passthrough_args(matches: &ArgMatches) -> Vec<OsString> {
    matches
        .get_many::<OsString>(PASSTHROUGH_ARGS)
        .map(|values| values.cloned().collect::<Vec<_>>())
        .unwrap_or_default()
}

pub fn parse_required_subcommand<I>(
    command: Command,
    args: I,
) -> Result<(String, Vec<OsString>), clap::Error>
where
    I: IntoIterator<Item = OsString>,
{
    let mut command = command.subcommand_required(true);
    let matches = parse_matches(command.clone(), args)?;
    let Some((name, matches)) = matches.subcommand() else {
        return Err(command.error(ErrorKind::MissingSubcommand, "a subcommand is required"));
    };

    Ok((name.to_string(), passthrough_args(matches)))
}

pub fn parse_required_subcommand_or_usage<I>(
    command: Command,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<(String, Vec<OsString>), String>
where
    I: IntoIterator<Item = OsString>,
{
    parse_required_subcommand(command, args).map_err(|_| usage())
}

pub fn parse_optional_subcommand_or_usage<I>(
    command: Command,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<OptionalSubcommand, String>
where
    I: IntoIterator<Item = OsString>,
{
    let matches = parse_matches_or_usage(command, args, usage)?;
    if let Some((name, matches)) = matches.subcommand() {
        return Ok(OptionalSubcommand::Matched {
            name: name.to_string(),
            args: passthrough_args(matches),
        });
    }

    Ok(OptionalSubcommand::Passthrough(passthrough_args(&matches)))
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

pub fn required_string(matches: &ArgMatches, id: &str) -> String {
    string_option(matches, id).unwrap_or_else(|| panic!("clap requires {id}"))
}

pub fn typed_option<T>(matches: &ArgMatches, id: &str) -> Option<T>
where
    T: Clone + Send + Sync + 'static,
{
    matches.get_one::<T>(id).cloned()
}

pub fn required_typed<T>(matches: &ArgMatches, id: &str) -> T
where
    T: Clone + Send + Sync + 'static,
{
    typed_option(matches, id).unwrap_or_else(|| panic!("clap requires {id}"))
}

pub fn render_help(mut command: Command) -> String {
    command.render_help().to_string()
}
