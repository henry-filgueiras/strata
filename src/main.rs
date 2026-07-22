use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use strata::cli::{ArtifactTarget, Cli, Collection, Command};
use strata::error::Error;
use strata::read::Status;
use strata::{artifact, doctor, fortune, read, repo, transition};

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
        Command::Close { reference } => transition(reference, Collection::Dragon, Status::Closed),
        Command::Reopen { reference } => transition(reference, Collection::Dragon, Status::Open),
        Command::Adopt { reference } => transition(reference, Collection::Idea, Status::Adopted),
        Command::Reject { reference } => transition(reference, Collection::Idea, Status::Rejected),
        Command::Fortune => fortune(),
    }
}

/// The read model of a command-line collection.
fn model(collection: Collection) -> &'static read::Collection {
    match collection {
        Collection::Dragon => &read::DRAGON,
        Collection::Idea => &read::IDEA,
    }
}

/// Convert a command-line artifact target into a read-model selector.
fn selector(target: &ArtifactTarget) -> read::Selector<'_> {
    match target {
        ArtifactTarget::Reference(reference) => read::Selector::Sequence(reference.sequence),
        ArtifactTarget::Id(id) => read::Selector::Id(id),
    }
}

/// Transition one artifact between lifecycle states and render the outcome.
///
/// Each transition verb belongs to one collection's lifecycle; a reference
/// into another collection is refused with the verbs that do apply, rather
/// than resolved into a surprising move.
fn transition(target: &ArtifactTarget, collection: Collection, to: Status) -> Result<(), Error> {
    if let ArtifactTarget::Reference(reference) = target
        && reference.collection != collection
    {
        let guidance = match reference.collection {
            Collection::Dragon => "dragons close and reopen: use `strata close` or `strata reopen`",
            Collection::Idea => "ideas adopt or reject: use `strata adopt` or `strata reject`",
        };
        return Err(Error::InvalidInvocation {
            message: format!(
                "`{target}` is a {} reference; {guidance}",
                reference.collection
            ),
        });
    }
    let root = repo::discover(&cwd()?)?;
    let done = transition::transition(
        &root,
        model(collection),
        selector(target),
        &target.to_string(),
        to,
    )?;
    let verb = match to {
        Status::Closed => "closed",
        Status::Open => "reopened",
        Status::Adopted => "adopted",
        Status::Rejected => "rejected",
        Status::Parked => "parked",
    };
    println!(
        "{verb} {} ({} -> {}) at {}",
        done.reference, done.from, done.to, done.to_path
    );
    Ok(())
}

/// Surface one open dragon or parked idea, weighted toward stale artifacts.
fn fortune() -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    let dragons = read::scan(&root, &read::DRAGON)?;
    let ideas = read::scan(&root, &read::IDEA)?;
    // The candidate pool is every artifact still owed attention: open
    // dragons and parked ideas. Terminal states never resurface.
    let pool: Vec<_> = dragons
        .iter()
        .filter(|artifact| artifact.summary.status == Status::Open)
        .chain(
            ideas
                .iter()
                .filter(|artifact| artifact.summary.status == Status::Parked),
        )
        .collect();
    if pool.is_empty() {
        println!(
            "no open dragons or parked ideas — nothing lurks; record a risk \
             with `strata new dragon \"<title>\"` or park a proposal with \
             `strata new idea \"<title>\"`"
        );
        return Ok(());
    }
    let today = jiff::Zoned::now().date();
    let ages: Vec<_> = pool
        .iter()
        .map(|artifact| fortune::age_days(&artifact.summary.created, today))
        .collect();
    let weights: Vec<u64> = ages.iter().map(|age| fortune::weight(*age)).collect();
    // The draw's entropy comes from a fresh ULID's 80-bit random component;
    // selection itself is the pure, tested `pick`.
    let index = fortune::pick(&weights, ulid::Ulid::new().random());
    let chosen = pool[index];
    println!("{}  {}", chosen.summary.reference(), chosen.summary.title);
    println!(
        "{}  {}",
        fortune::age_text(ages[index]),
        chosen.summary.path
    );
    let excerpt = fortune::excerpt(&chosen.content, 3);
    if !excerpt.is_empty() {
        println!();
        for line in &excerpt {
            println!("  {line}");
        }
    }
    Ok(())
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
    let created = match collection {
        Collection::Dragon => artifact::create_dragon(&root, title)?,
        Collection::Idea => artifact::create_idea(&root, title)?,
    };
    println!(
        "created {} at {}",
        created.reference(),
        created.relative_path.display()
    );
    Ok(())
}

/// List a collection's artifacts and render the requested projection.
fn list(collection: Collection, json: bool) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    let artifacts = read::scan(&root, model(collection))?;
    if json {
        let summaries: Vec<_> = artifacts.iter().map(|a| &a.summary).collect();
        println!("{}", to_json(&summaries));
    } else if artifacts.is_empty() {
        println!(
            "no {}s found; create one with `strata new {} \"<title>\"`",
            collection.name(),
            collection.name()
        );
    } else {
        for artifact in &artifacts {
            let summary = &artifact.summary;
            println!(
                "{}  {:<8}  {}  ({})",
                summary.reference(),
                summary.status,
                summary.title,
                summary.path
            );
        }
    }
    Ok(())
}

/// Resolve one artifact reference and render it.
///
/// A `collection:sequence` reference scans exactly that collection; a bare
/// stable id could live in any collection, so every managed collection is
/// scanned and the id resolved over the union.
fn show(target: &ArtifactTarget, json: bool) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    let artifacts = match target {
        ArtifactTarget::Reference(reference) => read::scan(&root, model(reference.collection))?,
        ArtifactTarget::Id(_) => {
            let mut all = read::scan(&root, &read::DRAGON)?;
            all.extend(read::scan(&root, &read::IDEA)?);
            all
        }
    };
    let artifact = read::resolve(&artifacts, selector(target), &target.to_string())?;
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
    } else {
        for finding in &report.findings {
            let prefix = match finding.severity {
                doctor::Severity::Error => "",
                doctor::Severity::Advice => "advice  ",
            };
            println!(
                "{prefix}{}  {}: {}",
                finding.problem, finding.path, finding.detail
            );
        }
        if report.healthy() {
            let advice = report.findings.len();
            if advice > 0 {
                println!(
                    "doctor: {} artifact(s) checked, no problems found, \
                     {advice} advisory note(s)",
                    report.artifacts_checked
                );
            } else {
                println!(
                    "doctor: {} artifact(s) checked, no problems found",
                    report.artifacts_checked
                );
            }
        }
    }
    if report.healthy() {
        Ok(())
    } else {
        Err(Error::UnhealthyRepository {
            problems: report.problems(),
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
