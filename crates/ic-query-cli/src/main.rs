//! Process entrypoint for specialist IC query adapters.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Command;

mod presentation;
mod topology;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut root = Command::new("icq")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Read-only IC infrastructure evidence for Canic")
        .subcommand(topology::command());
    root.build();
    let mut root = alphabetical(root);
    let matches = root.get_matches_mut();
    let Some(("topology", topology)) = matches.subcommand() else {
        root.print_help()?;
        return Ok(());
    };
    if topology.subcommand().is_none() {
        root.find_subcommand_mut("topology")
            .expect("declared namespace")
            .print_help()?;
        return Ok(());
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let output = runtime.block_on(topology::run(topology))?;
    writeln!(io::stdout().lock(), "{output}")?;
    Ok(())
}

// Apply one presentation policy after Clap has generated its help commands.
fn alphabetical(command: Command) -> Command {
    command.display_order(0).mut_subcommands(alphabetical)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("icq: {error}");
            ExitCode::FAILURE
        }
    }
}
