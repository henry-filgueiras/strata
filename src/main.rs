use std::process::ExitCode;

use clap::Parser;
use strata::cli::{Cli, Command};
use strata::error::Error;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error.render());
            ExitCode::from(error.exit_code())
        }
    }
}

/// Dispatch a parsed command.
///
/// Every command is a stub while the surface and error model land first;
/// each command's behavior arrives with its own sprint task.
fn run(command: &Command) -> Result<(), Error> {
    match command {
        Command::Init => Err(Error::unimplemented("init")),
        Command::New { .. } => Err(Error::unimplemented("new")),
        Command::List { .. } => Err(Error::unimplemented("list")),
        Command::Show { .. } => Err(Error::unimplemented("show")),
        Command::Doctor => Err(Error::unimplemented("doctor")),
    }
}
