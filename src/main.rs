use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use strata::cli::{Cli, Collection, Command, ShowTarget};
use strata::error::Error;
use strata::{artifact, doctor, read, repo};

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
fn run(command: &Command) -> Result<(), Error> {
    match command {
        Command::Init => init(),
        Command::New { collection, title } => new_artifact(*collection, title),
        Command::List { collection, json } => list(*collection, *json),
        Command::Show { reference, json } => show(reference, *json),
        Command::Doctor { json } => doctor(*json),
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

/// List a collection's artifacts and render the requested projection.
fn list(collection: Collection, json: bool) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    match collection {
        Collection::Dragon => {
            let artifacts = read::scan_dragons(&root)?;
            if json {
                let summaries: Vec<_> = artifacts.iter().map(|a| &a.summary).collect();
                println!("{}", to_json(&summaries));
            } else if artifacts.is_empty() {
                println!("no dragons found; create one with `strata new dragon \"<title>\"`");
            } else {
                for artifact in &artifacts {
                    let summary = &artifact.summary;
                    println!(
                        "{}  {:<6}  {}  ({})",
                        summary.reference(),
                        summary.status,
                        summary.title,
                        summary.path
                    );
                }
            }
        }
    }
    Ok(())
}

/// Resolve one artifact reference and render it.
fn show(target: &ShowTarget, json: bool) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    let artifacts = read::scan_dragons(&root)?;
    let dragon_ref = match target {
        ShowTarget::Reference(reference) => read::DragonRef::Sequence(reference.sequence),
        ShowTarget::Id(id) => read::DragonRef::Id(id),
    };
    let artifact = read::resolve(&artifacts, dragon_ref, &target.to_string())?;
    if json {
        println!("{}", to_json(&artifact.show_record()));
    } else {
        // The canonical file contents, byte-for-byte: no added newline.
        print!("{}", artifact.content);
    }
    Ok(())
}

/// Validate the enclosing repository and render every finding.
///
/// Findings are the stdout payload — human lines or a deterministic JSON
/// array — so `--json` output stays parseable even when validation fails;
/// an unhealthy repository is then reported through the error contract
/// (`unhealthy-repository`, exit code 9) on stderr.
fn doctor(json: bool) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    let report = doctor::check(&root)?;
    if json {
        println!("{}", to_json(&report.findings));
    } else if report.healthy() {
        println!(
            "doctor: {} artifact(s) checked, no problems found",
            report.artifacts_checked
        );
    } else {
        for finding in &report.findings {
            println!("{}  {}: {}", finding.problem, finding.path, finding.detail);
        }
    }
    if report.healthy() {
        Ok(())
    } else {
        Err(Error::UnhealthyRepository {
            problems: report.findings.len(),
        })
    }
}

/// Serialize a projection built from plain strings and integers.
fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("projections of plain data always serialize")
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
