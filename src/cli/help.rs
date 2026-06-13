use clap::Command;
use std::ffi::OsString;

use super::commands::COMMAND_FAMILIES;
use super::globals::network_arg;

const TOP_LEVEL_HELP_TEMPLATE: &str = "{name} {version}\n{about-with-newline}\n{usage-heading} {usage}\n\nCommands:\n{subcommands}\n\nOptions:\n{options}{after-help}\n";

fn is_help_arg(arg: &OsString) -> bool {
    arg.to_str()
        .is_some_and(|arg| matches!(arg, "help" | "--help" | "-h"))
}

fn is_version_arg(arg: &OsString) -> bool {
    arg.to_str()
        .is_some_and(|arg| matches!(arg, "version" | "--version" | "-V"))
}

pub fn first_arg_is_help(args: &[OsString]) -> bool {
    args.first().is_some_and(is_help_arg)
}

fn first_arg_is_version(args: &[OsString]) -> bool {
    args.first().is_some_and(is_version_arg)
}

pub fn print_help_or_version(
    args: &[OsString],
    usage: impl FnOnce() -> String,
    version_text: &str,
) -> bool {
    if first_arg_is_help(args) {
        println!("{}", usage());
        return true;
    }
    if first_arg_is_version(args) {
        println!("{version_text}");
        return true;
    }
    false
}

#[must_use]
pub fn top_level_command() -> Command {
    Command::new("icq")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Internet Computer metadata query CLI")
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg(
            clap::Arg::new("version")
                .short('V')
                .long("version")
                .action(clap::ArgAction::SetTrue)
                .help("Print version"),
        )
        .arg(network_arg().global(true))
        .subcommand_help_heading("Commands")
        .help_template(TOP_LEVEL_HELP_TEMPLATE)
        .after_help("Run `icq <command> help` for command-specific help.")
        .subcommands(
            COMMAND_FAMILIES
                .iter()
                .map(|family| Command::new(family.name).about(family.about)),
        )
}

pub fn usage() -> String {
    let mut command = top_level_command();
    command.render_help().to_string()
}
