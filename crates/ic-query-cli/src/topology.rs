//! Command adapter for the topology library.

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use ic_platform_types::Principal;

use crate::presentation;

pub(crate) fn command() -> Command {
    Command::new("topology")
        .about("Inspect infrastructure evidence for Canic placement")
        .subcommand(
            Command::new("canister")
                .about("Look up one canister's subnet in the mainnet NNS Registry")
                .arg(
                    Arg::new("canister")
                        .value_name("CANISTER_ID")
                        .required(true)
                        .value_parser(value_parser!(Principal)),
                )
                .arg(json_argument()),
        )
        .subcommand(
            Command::new("subnet")
                .about("Observe node identities in a subnet's certified state")
                .arg(
                    Arg::new("subnet")
                        .value_name("SUBNET_ID")
                        .required(true)
                        .value_parser(value_parser!(Principal)),
                )
                .arg(json_argument()),
        )
}

fn json_argument() -> Arg {
    Arg::new("json")
        .long("json")
        .help("Print the complete report as JSON")
        .action(ArgAction::SetTrue)
}

pub(crate) async fn run(matches: &ArgMatches) -> Result<String, Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("canister", arguments)) => {
            let id = *arguments
                .get_one::<Principal>("canister")
                .expect("required typed argument");
            let report = ic_topology::placement::fetch(id).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::placement(&report))
            }
        }
        Some(("subnet", arguments)) => {
            let id = *arguments
                .get_one::<Principal>("subnet")
                .expect("required typed argument");
            let report = ic_topology::subnet::fetch(id).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet(&report))
            }
        }
        _ => unreachable!(
            "Clap admits only declared operations; the entrypoint handles namespace help"
        ),
    }
}
