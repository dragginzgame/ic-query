//! Module: cli::clap
//!
//! Responsibility: small clap helper wrappers shared by command parsers.
//! Does not own: command-family specs, report requests, or runtime dispatch.
//! Boundary: normalizes typed values, parse errors, and help rendering.

use clap::{
    Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches,
    builder::{Resettable, TypedValueParser},
};
use std::ffi::OsString;

/// Prepare the complete Clap tree with shared namespace-help and alphabetical
/// subcommand-display policies.
#[must_use]
pub fn prepare_command_tree(command: Command) -> Command {
    let mut command = default_namespaces_to_help(command);
    command.build();
    set_equal_subcommand_display_order(command)
}

fn default_namespaces_to_help(command: Command) -> Command {
    let has_subcommands = command.has_subcommands();
    let command = command.mut_subcommands(default_namespaces_to_help);
    if has_subcommands {
        command
            .arg_required_else_help(true)
            .subcommand_required(false)
    } else {
        command
    }
}

fn set_equal_subcommand_display_order(command: Command) -> Command {
    command.mut_subcommands(|subcommand| {
        set_equal_subcommand_display_order(subcommand).display_order(0)
    })
}

pub fn parse_matches<I>(command: Command, args: I) -> Result<ArgMatches, clap::Error>
where
    I: IntoIterator<Item = OsString>,
{
    let name = command.get_name().to_string();
    command.try_get_matches_from(std::iter::once(OsString::from(name)).chain(args))
}

pub fn parsed_options<T: FromArgMatches>(matches: &ArgMatches) -> T {
    T::from_arg_matches(matches).expect("Clap admitted the command's typed argument schema")
}

/// Attach typed arguments without replacing command help with Rust type documentation.
pub fn with_options<T: Args>(command: Command) -> Command {
    let about = command
        .get_about()
        .cloned()
        .map_or(Resettable::Reset, Resettable::Value);
    let long_about = command
        .get_long_about()
        .cloned()
        .map_or(Resettable::Reset, Resettable::Value);
    T::augment_args(command).about(about).long_about(long_about)
}

pub fn flag_arg(id: &'static str) -> Arg {
    Arg::new(id).action(ArgAction::SetTrue)
}

#[cfg(test)]
pub use super::tests::render_help;

/// Parse and canonicalize a principal through Clap's typed parser.
pub fn principal_text_value_parser() -> impl TypedValueParser<Value = String> {
    clap::builder::StringValueParser::new().try_map(|value| {
        value
            .parse::<ic_platform::candid::contract::Principal>()
            .map(|principal| principal.to_text())
    })
}

/// Adapt a domain-owned choice list to Clap without duplicating its vocabulary.
pub fn choice_parser<T: Copy + Send + Sync + 'static>(
    choices: &'static [T],
    label: fn(T) -> &'static str,
) -> impl clap::builder::TypedValueParser<Value = T> {
    clap::builder::PossibleValuesParser::new(choices.iter().copied().map(label)).map(move |value| {
        choices
            .iter()
            .copied()
            .find(|choice| label(*choice) == value)
            .expect("Clap admitted a declared choice")
    })
}
