//! Process entrypoint for specialist IC query adapters.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Command;

mod presentation;
mod topology;

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut root = Command::new("icq")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Read-only IC infrastructure evidence for Canic")
        .subcommand(topology::command());
    root.build();
    let mut root = alphabetical(root);
    let matches = root.get_matches_mut();
    // Walk Clap's parsed tree so every namespace has the same local help behavior.
    let mut selected_command = &mut root;
    let mut selected_matches = &matches;
    while let Some((name, child_matches)) = selected_matches.subcommand() {
        selected_command = selected_command
            .find_subcommand_mut(name)
            .expect("Clap admits only declared commands");
        selected_matches = child_matches;
    }
    if selected_command.has_subcommands() {
        selected_command.print_help()?;
        return Ok(ExitCode::SUCCESS);
    }
    let (_, topology) = matches.subcommand().expect("declared specialist namespace");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let output = runtime.block_on(topology::run(topology))?;
    writeln!(io::stdout().lock(), "{}", output.text)?;
    Ok(output.exit_code)
}

// Apply one presentation policy after Clap has generated its help commands.
fn alphabetical(command: Command) -> Command {
    command.display_order(0).mut_subcommands(alphabetical)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("icq: {error}");
            ExitCode::FAILURE
        }
    }
}
