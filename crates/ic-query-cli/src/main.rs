//! Process entrypoint for specialist IC query adapters.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Command;

mod migration;
mod presentation;
mod topology;

pub(crate) struct CommandOutput {
    pub(crate) text: String,
    pub(crate) exit_code: ExitCode,
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut root = Command::new("icq")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Read-only IC topology and migration evidence")
        .subcommand(topology::command())
        .subcommand(migration::icrc_command())
        .subcommand(migration::sns_command());
    root.build();
    let mut root = help_style(root);
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
    let (namespace, selected) = matches.subcommand().expect("declared specialist namespace");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let output = if namespace == "topology" {
        runtime.block_on(topology::run(selected))?
    } else {
        runtime.block_on(migration::run(namespace, selected))?
    };
    writeln!(io::stdout().lock(), "{}", output.text)?;
    Ok(output.exit_code)
}

// Apply one presentation policy after Clap has generated its help commands.
fn help_style(command: Command) -> Command {
    command
        .display_order(0)
        .help_template(concat!(
            "IC Query ",
            env!("CARGO_PKG_VERSION"),
            " : {about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}"
        ))
        .mut_subcommands(help_style)
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
