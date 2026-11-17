//! Compile-time ICRC and SNS command adapters; no migration execution.
use crate::{CommandOutput, presentation::migration as render};
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use ic_platform_types::{Account, Principal};
use ic_sns_migration::{balance, neuron};
use std::process::ExitCode;

fn json() -> Arg {
    Arg::new("json")
        .long("json")
        .action(ArgAction::SetTrue)
        .help("Print raw schema-1 JSON evidence")
}
fn principal(name: &'static str) -> Arg {
    Arg::new(name)
        .required(true)
        .value_parser(value_parser!(Principal))
}
pub(crate) fn icrc_command() -> Command {
    Command::new("icrc")
        .about("Read selected ICRC accounts for migration evidence")
        .subcommand(
            Command::new("balance")
                .about("Read exact balance and independent metadata; partial evidence exits 1")
                .arg(principal("ledger"))
                .arg(principal("owner"))
                .arg(
                    Arg::new("subaccount")
                        .long("subaccount")
                        .value_parser(parse_hex32)
                        .help(
                            "Exactly 64 hexadecimal digits; omitted selects the default subaccount",
                        ),
                )
                .arg(json()),
        )
}
pub(crate) fn sns_command() -> Command {
    Command::new("sns").about("Read selected SNS neurons for migration evidence")
        .subcommand(Command::new("neuron").about("Inspect selected fields; not a full neuron export or entitlement decision")
            .subcommand(Command::new("show").about("Read one exact neuron from an explicit SNS Governance canister")
                .arg(principal("governance"))
                .arg(Arg::new("neuron-id").required(true).value_parser(parse_hex32)).arg(json()))
            .subcommand(Command::new("list").about("Read bounded pages; incomplete traversal exits 1; no atomic snapshot")
                .arg(principal("governance"))
                .arg(Arg::new("principal").long("principal").value_parser(value_parser!(Principal))
                    .conflicts_with_all(["max-pages", "start-after"]).help("Filter permission principal; one page only, not an ownership claim"))
                .arg(Arg::new("start-after").long("start-after").value_parser(parse_hex32)
                    .help("Continue after an exact 32-byte neuron ID (64 hex digits); no snapshot guarantee"))
                .arg(Arg::new("max-pages").long("max-pages").default_value("1")
                    .value_parser(value_parser!(u8).range(1..=i64::from(neuron::MAX_PAGES)))
                    .help("At most 100 neurons per page, 60 seconds total"))
                .arg(json())))
}
fn parse_hex32(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected exactly 64 hexadecimal digits (32 bytes)".into());
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).expect("validated ASCII hex");
    }
    Ok(bytes)
}
fn output(text: String, complete: bool) -> CommandOutput {
    CommandOutput {
        text,
        exit_code: if complete {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    }
}
pub(crate) async fn run(
    namespace: &str,
    matches: &ArgMatches,
) -> Result<CommandOutput, Box<dyn std::error::Error>> {
    if namespace == "icrc" {
        let (_, args) = matches.subcommand().expect("balance command");
        let report = balance::fetch(
            *args.get_one::<Principal>("ledger").expect("required"),
            Account {
                owner: *args.get_one::<Principal>("owner").expect("required"),
                subaccount: args.get_one::<[u8; 32]>("subaccount").copied(),
            },
        )
        .await?;
        let text = if args.get_flag("json") {
            serde_json::to_string_pretty(&report)?
        } else {
            render::balance(&report)
        };
        return Ok(output(text, report.complete));
    }
    let (_, neurons) = matches.subcommand().expect("neuron namespace");
    let (operation, args) = neurons.subcommand().expect("neuron operation");
    let governance = *args.get_one::<Principal>("governance").expect("required");
    if operation == "show" {
        let report = neuron::fetch(
            governance,
            *args.get_one::<[u8; 32]>("neuron-id").expect("required"),
        )
        .await?;
        let text = if args.get_flag("json") {
            serde_json::to_string_pretty(&report)?
        } else {
            render::neuron(&report)
        };
        Ok(output(text, report.neuron.is_read()))
    } else {
        let report = neuron::list(neuron::ListRequest {
            governance,
            of_principal: args.get_one::<Principal>("principal").copied(),
            start_after: args.get_one::<[u8; 32]>("start-after").copied(),
            max_pages: *args.get_one::<u8>("max-pages").expect("default"),
        })
        .await?;
        let text = if args.get_flag("json") {
            serde_json::to_string_pretty(&report)?
        } else {
            render::neurons(&report)
        };
        Ok(output(text, report.traversal_exhausted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filtered_default_and_continuation_are_admitted_without_io() {
        let gov = "zqfso-syaaa-aaaaq-aaafq-cai";
        let filtered = sns_command()
            .try_get_matches_from(["sns", "neuron", "list", gov, "--principal", "2vxsx-fae"])
            .unwrap();
        assert_eq!(
            *filtered
                .subcommand()
                .unwrap()
                .1
                .subcommand()
                .unwrap()
                .1
                .get_one::<u8>("max-pages")
                .unwrap(),
            1
        );
        let cursor = "AB".repeat(32);
        let continuation = sns_command()
            .try_get_matches_from(["sns", "neuron", "list", gov, "--start-after", &cursor])
            .unwrap();
        assert_eq!(
            *continuation
                .subcommand()
                .unwrap()
                .1
                .subcommand()
                .unwrap()
                .1
                .get_one::<[u8; 32]>("start-after")
                .unwrap(),
            [0xab; 32]
        );
        assert!(
            sns_command()
                .try_get_matches_from([
                    "sns",
                    "neuron",
                    "list",
                    gov,
                    "--start-after",
                    &cursor,
                    "--principal",
                    "2vxsx-fae"
                ])
                .is_err()
        );
    }
}
