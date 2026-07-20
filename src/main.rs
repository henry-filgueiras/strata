use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use strata::cli::{Cli, Collection, Command};
use strata::error::Error;
use strata::{artifact, repo};

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
        Command::New { collection, title } => new_artifact(*collection, title),
        Command::List { .. } => Err(Error::unimplemented("list")),
        Command::Show { .. } => Err(Error::unimplemented("show")),
        Command::Doctor => Err(Error::unimplemented("doctor")),
    }
}

/// Resolve the current working directory.
fn cwd() -> Result<PathBuf, Error> {
    std::env::current_dir().map_err(|source| Error::Filesystem {
        operation: "resolve current directory".into(),
        path: PathBuf::from("."),
        source,
    })
}

/// Create an artifact in the enclosing repository and render the outcome.
fn new_artifact(collection: Collection, title: &str) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    match collection {
        Collection::Dragon => {
            let dragon = artifact::create_dragon(&root, title)?;
            println!(
                "created {} at {}",
                dragon.reference(),
                dragon.relative_path.display()
            );
        }
    }
    Ok(())
}

/// Initialize the current working directory and render the outcome.
fn init() -> Result<(), Error> {
    let cwd = cwd()?;
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
