mod cli;
mod ic;
mod icrc;
mod nns;
mod output;
mod progress;
mod sns;
mod storage;
mod system;

#[cfg(test)]
mod test_support;

use crate::cli::clap::{
    parse_matches_or_usage, passthrough_args, passthrough_subcommand, string_option,
};
use clap::{Arg, ArgAction, Command};
use ic_query::subnet_catalog::MAINNET_NETWORK;
use std::ffi::OsString;
use thiserror::Error as ThisError;

const TOP_LEVEL_HELP_TEMPLATE: &str = "{name} {version}\n{about-with-newline}\n{usage-heading} {usage}\n\nCommands:\n{subcommands}\n\nOptions:\n{options}{after-help}\n";
const VERSION_TEXT: &str = concat!("icq ", env!("CARGO_PKG_VERSION"));
const INTERNAL_NETWORK_OPTION: &str = "--__icq-network";

const fn version_text() -> &'static str {
    VERSION_TEXT
}

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
    Nns(#[from] nns::NnsCommandError),

    #[error("icrc: {0}")]
    Icrc(#[from] icrc::IcrcCommandError),

    #[error("ic: {0}")]
    Ic(#[from] ic::IcCommandError),

    #[error("sns: {0}")]
    Sns(#[from] sns::SnsCommandError),

    #[error("system: {0}")]
    System(#[from] system::SystemCommandError),
}

impl IcqCliError {
    /// Whether stdout closed before the command finished writing its report.
    #[must_use]
    pub fn is_broken_pipe(&self) -> bool {
        match self {
            Self::Ic(ic::IcCommandError::Io(err))
            | Self::Nns(nns::NnsCommandError::Io(err))
            | Self::Icrc(icrc::IcrcCommandError::Io(err))
            | Self::Sns(sns::SnsCommandError::Io(err))
            | Self::System(system::SystemCommandError::Io(err)) => {
                err.kind() == std::io::ErrorKind::BrokenPipe
            }
            Self::Usage(_)
            | Self::Nns(_)
            | Self::Icrc(_)
            | Self::Ic(_)
            | Self::Sns(_)
            | Self::System(_) => false,
        }
    }

    /// Process exit code for this command error.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_)
            | Self::Ic(ic::IcCommandError::Usage(_))
            | Self::Nns(nns::NnsCommandError::Usage(_))
            | Self::Icrc(icrc::IcrcCommandError::Usage(_))
            | Self::Sns(sns::SnsCommandError::Usage(_))
            | Self::System(system::SystemCommandError::Usage(_)) => 2,
            Self::Nns(_) | Self::Icrc(_) | Self::Ic(_) | Self::Sns(_) | Self::System(_) => 1,
        }
    }
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
    if let Some((command, option)) = command_local_global_option(&args) {
        if matches!(command, "ic" | "icrc") {
            return Err(unsupported_global_network_error(command));
        }
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
    apply_global_network(command, &mut tail, global_network)?;
    let tail = tail.into_iter();

    match command {
        "ic" => Ok(ic::run(tail)?),
        "icrc" => Ok(icrc::run(tail)?),
        "nns" => Ok(nns::run(tail)?),
        "sns" => Ok(sns::run(tail)?),
        "system" => Ok(system::run(tail)?),
        _ => unreachable!("top-level dispatch command only defines known commands"),
    }
}

fn collect_args_or_print_help<I>(args: I, usage: impl FnOnce() -> String) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if top_level_help_requested(&args) {
        println!("{}", usage());
        return None;
    }
    Some(args)
}

fn top_level_help_requested(args: &[OsString]) -> bool {
    let mut index = 0;
    while index < args.len() {
        let Some(arg) = args[index].to_str() else {
            return false;
        };
        if command_family(arg).is_some() {
            return false;
        }
        if matches!(arg, "help" | "--help" | "-h") {
            return true;
        }
        index += if arg == "--network" { 2 } else { 1 };
    }
    false
}

fn network_arg() -> Arg {
    Arg::new("network")
        .num_args(1)
        .long("network")
        .value_name("name")
        .help("Network identity for NNS, SNS, and system commands; currently only ic")
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

fn command_local_global_option(args: &[OsString]) -> Option<(&'static str, &'static str)> {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_str()?;
        if let Some(family) = command_family(arg) {
            return args[index + 1..]
                .iter()
                .filter_map(|arg| arg.to_str())
                .find_map(global_option_name)
                .map(|option| (family.name, option));
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

fn apply_global_network(
    command: &str,
    tail: &mut Vec<OsString>,
    global_network: Option<String>,
) -> Result<(), IcqCliError> {
    let Some(global_network) = global_network else {
        return Ok(());
    };
    if tail_requests_help_or_version(tail) {
        return Ok(());
    }
    if !command_accepts_global_network(command, tail) {
        return Err(unsupported_global_network_error(command));
    }
    if global_network != MAINNET_NETWORK {
        return Err(unsupported_mainnet_network_error(command, &global_network));
    }
    if tail_has_option(tail, INTERNAL_NETWORK_OPTION) {
        return Ok(());
    }

    tail.push(OsString::from(INTERNAL_NETWORK_OPTION));
    tail.push(OsString::from(global_network));
    Ok(())
}

fn unsupported_global_network_error(command: &str) -> IcqCliError {
    let guidance = if matches!(command, "ic" | "icrc") {
        " use the command's --source-endpoint option to select its API endpoint"
    } else {
        ""
    };
    IcqCliError::Usage(format!(
        "--network is not supported by `icq {command}`;{guidance}\n\n{}",
        usage()
    ))
}

fn unsupported_mainnet_network_error(command: &str, network: &str) -> IcqCliError {
    IcqCliError::Usage(format!(
        "`icq {command}` currently supports only the mainnet `{MAINNET_NETWORK}` network; received `{network}`\n\n{}",
        usage()
    ))
}

fn command_accepts_global_network(command: &str, tail: &[OsString]) -> bool {
    command_family(command).is_some_and(|family| (family.accepts_global_network)(tail))
}

fn tail_has_option(tail: &[OsString], name: &str) -> bool {
    tail.iter().any(|arg| arg.to_str() == Some(name))
}

fn tail_requests_help_or_version(tail: &[OsString]) -> bool {
    tail.iter()
        .filter_map(|arg| arg.to_str())
        .any(|arg| matches!(arg, "help" | "--help" | "-h" | "--version" | "-V"))
}

#[derive(Clone, Copy, Debug)]
struct CommandFamily {
    name: &'static str,
    about: &'static str,
    accepts_global_network: fn(&[OsString]) -> bool,
}

const COMMAND_FAMILIES: &[CommandFamily] = &[
    CommandFamily {
        name: "ic",
        about: "Inspect official IC Dashboard metadata",
        accepts_global_network: ic_accepts_global_network,
    },
    CommandFamily {
        name: "icrc",
        about: "Inspect generic ICRC ledger and account metadata",
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
    CommandFamily {
        name: "system",
        about: "Inspect native IC system-canister metadata",
        accepts_global_network: system_accepts_global_network,
    },
];

fn command_family(name: &str) -> Option<&'static CommandFamily> {
    COMMAND_FAMILIES.iter().find(|family| family.name == name)
}

const fn nns_accepts_global_network(_tail: &[OsString]) -> bool {
    true
}

const fn ic_accepts_global_network(_tail: &[OsString]) -> bool {
    false
}

const fn icrc_accepts_global_network(_tail: &[OsString]) -> bool {
    false
}

const fn sns_accepts_global_network(_tail: &[OsString]) -> bool {
    true
}

const fn system_accepts_global_network(_tail: &[OsString]) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_lists_query_families() {
        let text = usage();

        assert!(text.contains("Usage: icq [OPTIONS] [COMMAND]"));
        assert!(text.contains("ic"));
        assert!(text.contains("Inspect official IC Dashboard metadata"));
        assert!(text.contains("icrc"));
        assert!(text.contains("Inspect generic ICRC ledger and account metadata"));
        assert!(text.contains("nns"));
        assert!(text.contains("Inspect NNS metadata"));
        assert!(text.contains("sns"));
        assert!(text.contains("Inspect SNS metadata"));
        assert!(text.contains("system"));
        assert!(text.contains("Inspect native IC system-canister metadata"));
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
  ic      Inspect official IC Dashboard metadata
  icrc    Inspect generic ICRC ledger and account metadata
  nns     Inspect NNS metadata
  sns     Inspect SNS metadata
  system  Inspect native IC system-canister metadata

Options:
  -V, --version         Print version
      --network <name>  Network identity for NNS, SNS, and system commands; currently only ic
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
            &["ic", "help"][..],
            &["ic", "canister", "help"],
            &["ic", "canister", "info", "help"],
            &["icrc", "help"],
            &["icrc", "ledger", "help"],
            &["icrc", "ledger", "token", "help"],
            &["icrc", "account", "help"],
            &["icrc", "account", "balance", "help"],
            &["icrc", "account", "allowance", "help"],
            &["icrc", "account", "transaction", "help"],
            &["icrc", "account", "transaction", "page", "help"],
            &["icrc", "account", "transaction", "list", "help"],
            &["icrc", "account", "transaction", "refresh", "help"],
            &["icrc", "account", "transaction", "cache", "help"],
            &["icrc", "account", "transaction", "cache", "status", "help"],
            &["icrc", "ledger", "index", "help"],
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
            &["sns", "proposal", "list", "help"],
            &["sns", "proposal", "info", "help"],
            &["sns", "proposal", "cache", "help"],
            &["sns", "proposal", "cache", "list", "help"],
            &["sns", "proposal", "cache", "status", "help"],
            &["sns", "proposal", "refresh", "help"],
            &["sns", "neuron", "help"],
            &["sns", "neuron", "list", "help"],
            &["sns", "neuron", "cache", "help"],
            &["sns", "neuron", "cache", "list", "help"],
            &["sns", "neuron", "cache", "status", "help"],
            &["sns", "neuron", "refresh", "help"],
            &["system", "help"],
            &["system", "xdr", "help"],
            &["system", "cycles", "help"],
        ] {
            assert_run_ok(args);
        }
    }

    #[test]
    fn version_flags_return_ok() {
        assert_eq!(VERSION_TEXT, concat!("icq ", env!("CARGO_PKG_VERSION")));
        assert!(run([OsString::from("--version")]).is_ok());
        assert!(run([OsString::from("ic"), OsString::from("--version")]).is_ok());
        assert!(run([OsString::from("icrc"), OsString::from("--version")]).is_ok());
        assert!(run([OsString::from("nns"), OsString::from("--version")]).is_ok());
        assert!(run([OsString::from("sns"), OsString::from("--version")]).is_ok());
        assert!(run([OsString::from("system"), OsString::from("--version")]).is_ok());
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

        apply_global_network("sns", &mut sns_info_tail, Some("ic".to_string()))
            .expect("SNS supports global network");

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
    fn typed_cli_errors_preserve_exit_and_broken_pipe_semantics() {
        for usage in [
            IcqCliError::Ic(ic::IcCommandError::Usage("bad input".to_string())),
            IcqCliError::Icrc(icrc::IcrcCommandError::Usage("bad input".to_string())),
            IcqCliError::System(system::SystemCommandError::Usage("bad input".to_string())),
        ] {
            assert_eq!(usage.exit_code(), 2);
            assert!(!usage.is_broken_pipe());
        }

        for broken_pipe in [
            IcqCliError::Ic(ic::IcCommandError::Io(std::io::Error::from(
                std::io::ErrorKind::BrokenPipe,
            ))),
            IcqCliError::Icrc(icrc::IcrcCommandError::Io(std::io::Error::from(
                std::io::ErrorKind::BrokenPipe,
            ))),
            IcqCliError::System(system::SystemCommandError::Io(std::io::Error::from(
                std::io::ErrorKind::BrokenPipe,
            ))),
        ] {
            assert_eq!(broken_pipe.exit_code(), 1);
            assert!(broken_pipe.is_broken_pipe());
        }
    }

    #[test]
    fn global_network_is_forwarded_to_networked_leaf_commands() {
        for (command, leaf) in [
            ("nns", "data-center"),
            ("nns", "governance"),
            ("nns", "neuron"),
            ("nns", "node"),
            ("nns", "node-operator"),
            ("nns", "node-provider"),
            ("nns", "proposal"),
            ("nns", "registry"),
            ("nns", "subnet"),
            ("nns", "topology"),
            ("sns", "canister"),
            ("sns", "info"),
            ("sns", "list"),
            ("sns", "neuron"),
            ("sns", "params"),
            ("sns", "proposal"),
            ("sns", "token"),
            ("system", "xdr"),
        ] {
            let mut tail = vec![OsString::from(leaf), OsString::from("list")];

            apply_global_network(command, &mut tail, Some("ic".to_string()))
                .expect("NNS, SNS, and system families support the global network");

            assert_eq!(
                tail,
                vec![
                    OsString::from(leaf),
                    OsString::from("list"),
                    OsString::from(INTERNAL_NETWORK_OPTION),
                    OsString::from("ic")
                ]
            );
        }
    }

    #[test]
    fn non_mainnet_network_is_rejected_before_mainnet_only_dispatch() {
        for command in ["nns", "sns", "system"] {
            let mut tail = if command == "nns" {
                vec![OsString::from("proposal"), OsString::from("list")]
            } else {
                vec![OsString::from("list")]
            };

            let error = apply_global_network(command, &mut tail, Some("local".to_string()))
                .expect_err("current mainnet canister adapters reject other networks");

            assert_eq!(error.exit_code(), 2);
            assert!(error.to_string().contains("supports only the mainnet `ic`"));
            assert!(error.to_string().contains("received `local`"));
            assert!(!tail_has_option(&tail, INTERNAL_NETWORK_OPTION));
        }

        let mut preforwarded_tail = vec![
            OsString::from("proposal"),
            OsString::from("list"),
            OsString::from(INTERNAL_NETWORK_OPTION),
            OsString::from(MAINNET_NETWORK),
        ];
        let error = apply_global_network("nns", &mut preforwarded_tail, Some("local".to_string()))
            .expect_err("an internal forwarded value must not bypass global validation");
        assert!(error.to_string().contains("received `local`"));

        for args in [
            vec![
                OsString::from("--network"),
                OsString::from("local"),
                OsString::from("nns"),
                OsString::from("proposal"),
                OsString::from("list"),
            ],
            vec![
                OsString::from("--network"),
                OsString::from("local"),
                OsString::from("nns"),
                OsString::from("governance"),
                OsString::from("economics"),
            ],
            vec![
                OsString::from("--network"),
                OsString::from("local"),
                OsString::from("nns"),
                OsString::from("neuron"),
                OsString::from("list"),
            ],
            vec![
                OsString::from("--network"),
                OsString::from("local"),
                OsString::from("sns"),
                OsString::from("list"),
            ],
            vec![
                OsString::from("--network"),
                OsString::from("local"),
                OsString::from("sns"),
                OsString::from("canister"),
                OsString::from("list"),
                OsString::from("1"),
            ],
            vec![
                OsString::from("--network"),
                OsString::from("local"),
                OsString::from("system"),
                OsString::from("xdr"),
            ],
        ] {
            let command = args[2].to_string_lossy().into_owned();
            let error = run(args).expect_err("non-mainnet network must fail before dispatch");

            assert_eq!(error.exit_code(), 2);
            assert!(
                error
                    .to_string()
                    .starts_with(&format!("`icq {command}` currently"))
            );
        }
    }

    #[test]
    fn global_network_is_rejected_when_the_family_uses_endpoint_identity() {
        for (command, mut tail) in [
            (
                "ic",
                vec![OsString::from("canister"), OsString::from("info")],
            ),
            (
                "icrc",
                vec![OsString::from("ledger"), OsString::from("token")],
            ),
        ] {
            let original = tail.clone();
            let error = apply_global_network(command, &mut tail, Some("ic".to_string()))
                .expect_err("endpoint-identified family must reject global network");

            assert_eq!(error.exit_code(), 2);
            assert!(error.to_string().contains("--network is not supported"));
            assert!(error.to_string().contains(&format!("icq {command}")));
            assert!(error.to_string().contains("--source-endpoint"));
            assert_eq!(tail, original);
        }

        let error = run([
            OsString::from("--network"),
            OsString::from("ic"),
            OsString::from("icrc"),
            OsString::from("ledger"),
            OsString::from("token"),
            OsString::from("ryjl3-tyaaa-aaaaa-aaaba-cai"),
        ])
        .expect_err("ICRC global network must fail before dispatch");

        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("--source-endpoint"));

        let error = run([
            OsString::from("icrc"),
            OsString::from("ledger"),
            OsString::from("token"),
            OsString::from("ryjl3-tyaaa-aaaaa-aaaba-cai"),
            OsString::from("--network"),
            OsString::from("ic"),
        ])
        .expect_err("command-local ICRC network must use the same rejection");

        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("--network is not supported"));
        assert!(!error.to_string().contains("put it before the command"));

        assert!(
            run([
                OsString::from("--network"),
                OsString::from("ic"),
                OsString::from("icrc"),
                OsString::from("ledger"),
                OsString::from("token"),
                OsString::from("help"),
            ])
            .is_ok(),
            "help must remain available without dispatching a query"
        );

        let error = run([
            OsString::from("--network"),
            OsString::from("ic"),
            OsString::from("ic"),
            OsString::from("canister"),
            OsString::from("info"),
            OsString::from("ryjl3-tyaaa-aaaaa-aaaba-cai"),
        ])
        .expect_err("Dashboard family global network must fail before dispatch");

        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("icq ic"));
        assert!(error.to_string().contains("--source-endpoint"));

        let error = run([
            OsString::from("ic"),
            OsString::from("canister"),
            OsString::from("info"),
            OsString::from("ryjl3-tyaaa-aaaaa-aaaba-cai"),
            OsString::from("--network"),
            OsString::from("ic"),
        ])
        .expect_err("command-local Dashboard network must use the same rejection");

        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("--network is not supported"));
        assert!(!error.to_string().contains("put it before the command"));
    }

    #[test]
    fn malformed_source_endpoint_returns_typed_error_without_network_io() {
        let error = run([
            OsString::from("icrc"),
            OsString::from("ledger"),
            OsString::from("token"),
            OsString::from("ryjl3-tyaaa-aaaaa-aaaba-cai"),
            OsString::from("--source-endpoint"),
            OsString::from(":::"),
        ])
        .expect_err("malformed endpoint must return an error");

        assert_eq!(error.exit_code(), 1);
        assert!(error.to_string().contains("failed to build IC agent"));
        assert!(error.to_string().contains(":::"));
    }

    #[test]
    fn sns_nested_commands_dispatch_through_clap_subcommands() {
        assert!(
            run([
                OsString::from("sns"),
                OsString::from("neuron"),
                OsString::from("refresh"),
                OsString::from("--help")
            ])
            .is_ok()
        );
        assert!(
            run([
                OsString::from("sns"),
                OsString::from("proposal"),
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
