use crate::cli::clap::{passthrough_subcommand, value_arg};
use clap::{Arg, ArgAction, Command};
use std::ffi::OsString;

pub const DISPATCH_ARGS: &str = "args";
pub const INTERNAL_NETWORK_OPTION: &str = "--__icq-network";

const COMMANDS: &[&str] = &["nns", "sns"];

pub fn network_arg() -> Arg {
    value_arg("network")
        .long("network")
        .value_name("name")
        .help("ICP CLI network for networked commands")
}

pub fn internal_network_arg() -> Arg {
    value_arg("network").long("__icq-network").hide(true)
}

pub fn top_level_dispatch_command() -> Command {
    let command = Command::new("icq")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::SetTrue),
        )
        .arg(network_arg().global(true));

    COMMANDS.iter().fold(command, |command, name| {
        command.subcommand(passthrough_subcommand(Command::new(*name)))
    })
}

pub fn command_local_global_option(args: &[OsString]) -> Option<&'static str> {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_str()?;
        if COMMANDS.contains(&arg) {
            return args[index + 1..]
                .iter()
                .filter_map(|arg| arg.to_str())
                .find_map(global_option_name);
        }
        index += if arg == "--network" { 2 } else { 1 };
    }
    None
}

fn global_option_name(arg: &str) -> Option<&'static str> {
    match arg {
        "--network" => Some("--network"),
        _ if arg.starts_with("--network=") => Some("--network"),
        _ => None,
    }
}

pub fn apply_global_network(
    command: &str,
    tail: &mut Vec<OsString>,
    global_network: Option<String>,
) {
    let Some(global_network) = global_network else {
        return;
    };
    if tail_has_option(tail, INTERNAL_NETWORK_OPTION) {
        return;
    }
    if !command_accepts_global_network(command, tail) {
        return;
    }

    tail.push(OsString::from(INTERNAL_NETWORK_OPTION));
    tail.push(OsString::from(global_network));
}

fn command_accepts_global_network(command: &str, tail: &[OsString]) -> bool {
    match command {
        "nns" => nns_leaf_accepts_global_network(tail),
        "sns" => sns_accepts_global_network(tail),
        _ => false,
    }
}

fn nns_leaf_accepts_global_network(tail: &[OsString]) -> bool {
    matches!(
        tail.first().and_then(|arg| arg.to_str()),
        Some(
            "data-center"
                | "node"
                | "node-operator"
                | "node-provider"
                | "registry"
                | "subnet"
                | "topology"
        )
    )
}

fn sns_accepts_global_network(tail: &[OsString]) -> bool {
    matches!(
        tail.first().and_then(|arg| arg.to_str()),
        Some("list" | "info")
    )
}

fn tail_has_option(tail: &[OsString], name: &str) -> bool {
    tail.iter().any(|arg| arg.to_str() == Some(name))
}
