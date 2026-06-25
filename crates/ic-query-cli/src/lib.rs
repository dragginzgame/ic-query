use clap::{Arg, ArgAction, ArgMatches, Command};
use ic_query::{icrc, nns, sns};
use std::ffi::OsString;
use thiserror::Error as ThisError;

const PASSTHROUGH_ARGS: &str = "args";
const TOP_LEVEL_HELP_TEMPLATE: &str = "{name} {version}\n{about-with-newline}\n{usage-heading} {usage}\n\nCommands:\n{subcommands}\n\nOptions:\n{options}{after-help}\n";
const VERSION_TEXT: &str = concat!("icq ", env!("CARGO_PKG_VERSION"));
const INTERNAL_NETWORK_OPTION: &str = "--__icq-network";

///
/// IcqCliError
///
/// Top-level CLI dispatch error.
///

#[derive(Debug, ThisError)]
pub enum IcqCliError {
    #[error("{0}")]
    Usage(String),

    #[error("nns: {0}")]
    Nns(String),

    #[error("icrc: {0}")]
    Icrc(String),

    #[error("sns: {0}")]
    Sns(String),
}

/// Run the CLI from process arguments.
pub fn run_from_env() -> Result<(), IcqCliError> {
    run(std::env::args_os().skip(1))
}

/// Run the CLI from an argument iterator.
pub fn run<I>(args: I) -> Result<(), IcqCliError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = collect_args_or_print_help(args, usage) else {
        return Ok(());
    };
    if let Some(option) = command_local_global_option(&args) {
        return Err(IcqCliError::Usage(format!(
            "{option} is a top-level option; put it before the command\n\n{}",
            usage()
        )));
    }

    let matches = parse_matches_or_usage(top_level_dispatch_command(), args, usage)
        .map_err(IcqCliError::Usage)?;
    if matches.get_flag("version") {
        println!("{VERSION_TEXT}");
        return Ok(());
    }
    let global_network = string_option(&matches, "network");

    let Some((command, subcommand_matches)) = matches.subcommand() else {
        return Err(IcqCliError::Usage(usage()));
    };
    let mut tail = passthrough_args(subcommand_matches);
    apply_global_network(command, &mut tail, global_network);
    let tail = tail.into_iter();

    match command {
        "icrc" => icrc::run(tail).map_err(|err| IcqCliError::Icrc(err.to_string())),
        "nns" => nns::run(tail).map_err(|err| IcqCliError::Nns(err.to_string())),
        "sns" => sns::run(tail).map_err(|err| IcqCliError::Sns(err.to_string())),
        _ => unreachable!("top-level dispatch command only defines known commands"),
    }
}

fn parse_matches<I>(command: Command, args: I) -> Result<ArgMatches, clap::Error>
where
    I: IntoIterator<Item = OsString>,
{
    let name = command.get_name().to_string();
    command.try_get_matches_from(std::iter::once(OsString::from(name)).chain(args))
}

fn parse_matches_or_usage<I>(
    command: Command,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<ArgMatches, String>
where
    I: IntoIterator<Item = OsString>,
{
    parse_matches(command, args).map_err(|_| usage())
}

fn passthrough_subcommand(command: Command) -> Command {
    command.arg(
        Arg::new(PASSTHROUGH_ARGS)
            .num_args(0..)
            .allow_hyphen_values(true)
            .trailing_var_arg(true)
            .value_parser(clap::value_parser!(OsString)),
    )
}

fn passthrough_args(matches: &ArgMatches) -> Vec<OsString> {
    matches
        .get_many::<OsString>(PASSTHROUGH_ARGS)
        .map(|values| values.cloned().collect::<Vec<_>>())
        .unwrap_or_default()
}

fn string_option(matches: &ArgMatches, id: &str) -> Option<String> {
    matches.get_one::<String>(id).cloned()
}

fn collect_args_or_print_help<I>(args: I, usage: impl FnOnce() -> String) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if first_arg_is_help(&args) {
        println!("{}", usage());
        return None;
    }
    Some(args)
}

fn first_arg_is_help(args: &[OsString]) -> bool {
    args.first().is_some_and(|arg| {
        arg.to_str()
            .is_some_and(|arg| matches!(arg, "help" | "--help" | "-h"))
    })
}

fn network_arg() -> Arg {
    Arg::new("network")
        .num_args(1)
        .long("network")
        .value_name("name")
        .help("ICP CLI network for networked commands")
}

fn top_level_command() -> Command {
    Command::new("icq")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Internet Computer metadata query CLI")
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::SetTrue)
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

fn top_level_dispatch_command() -> Command {
    let command = Command::new("icq")
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::SetTrue),
        )
        .arg(network_arg().global(true));

    COMMAND_FAMILIES.iter().fold(command, |command, family| {
        command.subcommand(passthrough_subcommand(
            Command::new(family.name).about(family.about),
        ))
    })
}

fn usage() -> String {
    let mut command = top_level_command();
    command.render_help().to_string()
}

fn command_local_global_option(args: &[OsString]) -> Option<&'static str> {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_str()?;
        if command_family(arg).is_some() {
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

fn apply_global_network(command: &str, tail: &mut Vec<OsString>, global_network: Option<String>) {
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
    command_family(command).is_some_and(|family| (family.accepts_global_network)(tail))
}

fn tail_has_option(tail: &[OsString], name: &str) -> bool {
    tail.iter().any(|arg| arg.to_str() == Some(name))
}

#[derive(Clone, Copy, Debug)]
struct CommandFamily {
    name: &'static str,
    about: &'static str,
    accepts_global_network: fn(&[OsString]) -> bool,
}

const COMMAND_FAMILIES: &[CommandFamily] = &[
    CommandFamily {
        name: "icrc",
        about: "Inspect generic ICRC ledger metadata",
        accepts_global_network: icrc_accepts_global_network,
    },
    CommandFamily {
        name: "nns",
        about: "Inspect NNS metadata",
        accepts_global_network: nns_accepts_global_network,
    },
    CommandFamily {
        name: "sns",
        about: "Inspect SNS metadata",
        accepts_global_network: sns_accepts_global_network,
    },
];

fn command_family(name: &str) -> Option<&'static CommandFamily> {
    COMMAND_FAMILIES.iter().find(|family| family.name == name)
}

fn nns_accepts_global_network(tail: &[OsString]) -> bool {
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

const fn icrc_accepts_global_network(_tail: &[OsString]) -> bool {
    false
}

fn sns_accepts_global_network(tail: &[OsString]) -> bool {
    matches!(
        tail.first().and_then(|arg| arg.to_str()),
        Some("list" | "info" | "token" | "params" | "proposal" | "proposals" | "neurons")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_lists_query_families() {
        let text = usage();

        assert!(text.contains("Usage: icq [OPTIONS] [COMMAND]"));
        assert!(text.contains("icrc"));
        assert!(text.contains("Inspect generic ICRC ledger metadata"));
        assert!(text.contains("nns"));
        assert!(text.contains("Inspect NNS metadata"));
        assert!(text.contains("sns"));
        assert!(text.contains("Inspect SNS metadata"));
        assert!(text.contains("Run `icq <command> help`"));
    }

    #[test]
    fn top_level_usage_snapshot() {
        let expected = format!(
            "\
icq {}
Internet Computer metadata query CLI

Usage: icq [OPTIONS] [COMMAND]

Commands:
  icrc  Inspect generic ICRC ledger metadata
  nns   Inspect NNS metadata
  sns   Inspect SNS metadata

Options:
  -V, --version         Print version
      --network <name>  ICP CLI network for networked commands
  -h, --help            Print help

Run `icq <command> help` for command-specific help.
",
            env!("CARGO_PKG_VERSION")
        );

        assert_eq!(usage(), expected);
    }

    #[test]
    fn command_family_help_returns_ok() {
        for args in [
            &["icrc", "help"][..],
            &["icrc", "token", "help"],
            &["icrc", "balance", "help"],
            &["icrc", "allowance", "help"],
            &["icrc", "index", "help"],
            &["nns", "help"][..],
            &["nns", "data-center", "help"],
            &["nns", "data-center", "list", "help"],
            &["nns", "data-center", "info", "help"],
            &["nns", "data-center", "refresh", "help"],
            &["nns", "node", "help"],
            &["nns", "node", "list", "help"],
            &["nns", "node", "info", "help"],
            &["nns", "node", "refresh", "help"],
            &["nns", "node-provider", "help"],
            &["nns", "node-provider", "list", "help"],
            &["nns", "node-provider", "info", "help"],
            &["nns", "node-provider", "refresh", "help"],
            &["nns", "node-operator", "help"],
            &["nns", "node-operator", "list", "help"],
            &["nns", "node-operator", "info", "help"],
            &["nns", "node-operator", "refresh", "help"],
            &["nns", "proposal", "help"],
            &["nns", "proposal", "list", "help"],
            &["nns", "proposal", "info", "help"],
            &["nns", "registry", "help"],
            &["nns", "registry", "version", "help"],
            &["nns", "subnet", "help"],
            &["nns", "subnet", "list", "help"],
            &["nns", "subnet", "info", "help"],
            &["nns", "subnet", "refresh", "help"],
            &["nns", "topology", "help"],
            &["nns", "topology", "summary", "help"],
            &["nns", "topology", "coverage", "help"],
            &["nns", "topology", "versions", "help"],
            &["nns", "topology", "health", "help"],
            &["nns", "topology", "gaps", "help"],
            &["nns", "topology", "capacity", "help"],
            &["nns", "topology", "regions", "help"],
            &["nns", "topology", "providers", "help"],
            &["nns", "topology", "refresh", "help"],
            &["sns", "help"],
            &["sns", "list", "help"],
            &["sns", "info", "help"],
            &["sns", "token", "help"],
            &["sns", "params", "help"],
            &["sns", "proposal", "help"],
            &["sns", "proposals", "help"],
            &["sns", "neurons", "help"],
            &["sns", "neurons", "cache", "help"],
            &["sns", "neurons", "cache", "list", "help"],
            &["sns", "neurons", "cache", "status", "help"],
            &["sns", "neurons", "refresh", "help"],
        ] {
            assert_run_ok(args);
        }
    }

    #[test]
    fn version_flags_return_ok() {
        assert_eq!(VERSION_TEXT, concat!("icq ", env!("CARGO_PKG_VERSION")));
        assert!(run([OsString::from("--version")]).is_ok());
        assert!(run([OsString::from("icrc"), OsString::from("--version")]).is_ok());
        assert!(run([OsString::from("nns"), OsString::from("--version")]).is_ok());
        assert!(run([OsString::from("sns"), OsString::from("--version")]).is_ok());
        assert!(
            run([
                OsString::from("nns"),
                OsString::from("subnet"),
                OsString::from("list"),
                OsString::from("--version")
            ])
            .is_ok()
        );

        let mut sns_info_tail = vec![OsString::from("info"), OsString::from("1")];

        apply_global_network("sns", &mut sns_info_tail, Some("ic".to_string()));

        assert_eq!(
            sns_info_tail,
            vec![
                OsString::from("info"),
                OsString::from("1"),
                OsString::from(INTERNAL_NETWORK_OPTION),
                OsString::from("ic")
            ]
        );
    }

    #[test]
    fn global_network_is_forwarded_to_networked_leaf_commands() {
        let mut nns_tail = vec![OsString::from("data-center"), OsString::from("list")];

        apply_global_network("nns", &mut nns_tail, Some("ic".to_string()));

        assert_eq!(
            nns_tail,
            vec![
                OsString::from("data-center"),
                OsString::from("list"),
                OsString::from(INTERNAL_NETWORK_OPTION),
                OsString::from("ic")
            ]
        );

        let mut sns_tail = vec![OsString::from("list")];

        apply_global_network("sns", &mut sns_tail, Some("ic".to_string()));

        assert_eq!(
            sns_tail,
            vec![
                OsString::from("list"),
                OsString::from(INTERNAL_NETWORK_OPTION),
                OsString::from("ic")
            ]
        );

        let mut icrc_tail = vec![OsString::from("token")];

        apply_global_network("icrc", &mut icrc_tail, Some("ic".to_string()));

        assert_eq!(icrc_tail, vec![OsString::from("token")]);
    }

    #[test]
    fn sns_nested_commands_dispatch_through_clap_subcommands() {
        assert!(
            run([
                OsString::from("sns"),
                OsString::from("neurons"),
                OsString::from("refresh"),
                OsString::from("--help")
            ])
            .is_ok()
        );
        assert!(
            run([
                OsString::from("sns"),
                OsString::from("proposals"),
                OsString::from("cache"),
                OsString::from("status"),
                OsString::from("--help")
            ])
            .is_ok()
        );
    }

    fn assert_run_ok(args: &[&str]) {
        let args = args.iter().copied().map(OsString::from).collect::<Vec<_>>();
        if let Err(err) = run(args.clone()) {
            panic!("expected {args:?} to succeed, got {err}");
        }
    }
}
