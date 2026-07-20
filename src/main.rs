use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use strata::cli::{Cli, Command};
use strata::error::Error;
use strata::repo;

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
/// Remaining commands are stubs; each one's behavior arrives with its own
/// sprint task.
fn run(command: &Command) -> Result<(), Error> {
    match command {
        Command::Init => init(),
        Command::New { .. } => Err(Error::unimplemented("new")),
        Command::List { .. } => Err(Error::unimplemented("list")),
        Command::Show { .. } => Err(Error::unimplemented("show")),
        Command::Doctor => Err(Error::unimplemented("doctor")),
    }
}

/// Initialize the current working directory and render the outcome.
fn init() -> Result<(), Error> {
    let cwd = std::env::current_dir().map_err(|source| Error::Filesystem {
        operation: "resolve current directory".into(),
        path: PathBuf::from("."),
        source,
    })?;
    let report = repo::init(&cwd)?;
    if report.already_initialized() {
        println!(
            "Strata repository at `{}` is already initialized; nothing to change",
            cwd.display()
        );
    } else {
        println!("initialized Strata repository at `{}`", cwd.display());
        for path in &report.created {
            println!("  created {}", path.display());
        }
    }
    Ok(())
}
