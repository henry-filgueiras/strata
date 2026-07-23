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
        Command::New {
            collection,
            title,
            json,
        } => new_artifact(*collection, title, *json),
        Command::List {
            collection,
            json,
            active,
        } => list(*collection, *json, *active),
        Command::Show { reference, json } => show(reference, *json),
        Command::Doctor { json } => doctor(*json),
        Command::Close {
            reference,
            resolved_by,
        } => close(reference, resolved_by.as_deref()),
        Command::Reopen { reference } => transition(reference, Collection::Dragon, Status::Open),
        Command::Adopt {
            reference,
            adopted_by,
        } => adopt(reference, adopted_by.as_deref()),
        Command::Reject { reference } => transition(reference, Collection::Idea, Status::Rejected),
        Command::Fortune => fortune(),
    }
}

/// Scan one command-line collection into the shared read model.
fn scan(root: &std::path::Path, collection: Collection) -> Result<Vec<read::Artifact>, Error> {
    match collection {
        Collection::Dragon => read::scan(root, &read::DRAGON),
        Collection::Idea => read::scan(root, &read::IDEA),
        Collection::Sprint => read::scan_sprints(root),
        Collection::Task => read::scan_tasks(root),
    }
}

/// Convert a command-line artifact target into a read-model selector.
fn selector(target: &ArtifactTarget) -> read::Selector<'_> {
    match target {
        ArtifactTarget::Reference(reference) => read::Selector::Sequence(reference.sequence),
        ArtifactTarget::Id(id) => read::Selector::Id(id),
    }
}

/// The transition-verb guidance for one collection, used when a verb is
/// applied to a reference outside its lifecycle.
fn verb_guidance(collection: Collection) -> &'static str {
    match collection {
        Collection::Dragon => "dragons close and reopen: use `strata close` or `strata reopen`",
        Collection::Idea => "ideas adopt or reject: use `strata adopt` or `strata reject`",
        Collection::Sprint => "sprints close: use `strata close`",
        Collection::Task => "tasks close: use `strata close`",
    }
}

/// Transition one artifact between lifecycle states and render the outcome.
///
/// Each transition verb belongs to one collection's lifecycle; a reference
/// into another collection is refused with the verbs that do apply, rather
/// than resolved into a surprising rewrite.
fn transition(target: &ArtifactTarget, collection: Collection, to: Status) -> Result<(), Error> {
    if let ArtifactTarget::Reference(reference) = target
        && reference.collection != collection
    {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`{target}` is a {} reference; {}",
                reference.collection,
                verb_guidance(reference.collection)
            ),
        });
    }
    let root = repo::discover(&cwd()?)?;
    let done = transition::transition(
        &root,
        match collection {
            Collection::Dragon => &read::DRAGON,
            Collection::Idea => &read::IDEA,
            Collection::Sprint => &read::SPRINT,
            Collection::Task => &read::TASK,
        },
        selector(target),
        &target.to_string(),
        to,
    )?;
    render_transition(&done);
    Ok(())
}

/// `strata adopt`: an idea transition that may carry its `adopted-by`
/// provenance in the same invocation.
fn adopt(target: &ArtifactTarget, adopted_by: Option<&str>) -> Result<(), Error> {
    if let ArtifactTarget::Reference(reference) = target
        && reference.collection != Collection::Idea
    {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`{target}` is a {} reference; {}",
                reference.collection,
                verb_guidance(reference.collection)
            ),
        });
    }
    let root = repo::discover(&cwd()?)?;
    let done = transition::transition_with_provenance(
        &root,
        &read::IDEA,
        selector(target),
        &target.to_string(),
        Status::Adopted,
        adopted_by.map(|raw| ("adopted-by", raw)),
    )?;
    render_transition(&done);
    Ok(())
}

/// `strata close`: dragons and sprints share the verb, so the reference's
/// collection picks the lifecycle; a bare stable id resolves over the
/// union of the closable collections. `--resolved-by` records dragon
/// resolution provenance and belongs to no other collection's vocabulary.
fn close(target: &ArtifactTarget, resolved_by: Option<&str>) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    let collection = match target {
        ArtifactTarget::Reference(reference) => reference.collection,
        ArtifactTarget::Id(id) => {
            let union = || -> Result<Vec<read::Artifact>, Error> {
                let mut all = read::scan(&root, &read::DRAGON)?;
                all.extend(read::scan_sprints(&root)?);
                all.extend(read::scan_tasks(&root)?);
                Ok(all)
            };
            let union = union().map_err(|err| err.blocking(id))?;
            let artifact = read::resolve(&union, read::Selector::Id(id), id)?;
            match artifact.summary.kind.as_str() {
                "sprint" => Collection::Sprint,
                "task" => Collection::Task,
                _ => Collection::Dragon,
            }
        }
    };
    if resolved_by.is_some() && collection != Collection::Dragon {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`--resolved-by` records dragon resolution provenance; the \
                 decided vocabulary has no such edge for {}s",
                collection.name()
            ),
        });
    }
    let done = match collection {
        Collection::Dragon => transition::transition_with_provenance(
            &root,
            &read::DRAGON,
            selector(target),
            &target.to_string(),
            Status::Closed,
            resolved_by.map(|raw| ("resolved-by", raw)),
        )?,
        Collection::Sprint => {
            transition::close_sprint(&root, selector(target), &target.to_string())?
        }
        Collection::Task => transition::transition(
            &root,
            &read::TASK,
            selector(target),
            &target.to_string(),
            Status::Closed,
        )?,
        Collection::Idea => {
            return Err(Error::InvalidInvocation {
                message: format!(
                    "`{target}` is an idea reference; {}",
                    verb_guidance(Collection::Idea)
                ),
            });
        }
    };
    render_transition(&done);
    Ok(())
}

/// Render one performed transition.
fn render_transition(done: &transition::Transition) {
    let verb = match done.to {
        Status::Closed => "closed",
        Status::Open => "reopened",
        Status::Adopted => "adopted",
        Status::Rejected => "rejected",
        Status::Parked => "parked",
        Status::Active => "activated",
        Status::Pending => "pended",
    };
    println!(
        "{verb} {} ({} -> {}) at {}",
        done.reference, done.from, done.to, done.path
    );
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
///
/// Human and JSON projections consume the same semantic result: the
/// created artifact plus its decision 13 reachability. Flat creation
/// (dragons, ideas) runs the observational post-write probe; sprint and
/// task creation performed their strict containment scans before
/// writing, so a successful write is reachable by construction. A
/// degraded creation stays exit 0 — the write happened — with the stable
/// `warning[degraded-repository]:` line on stderr, leaving stdout (human
/// line or JSON object) unpolluted.
fn new_artifact(collection: Collection, title: &str, json: bool) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    let created = match collection {
        Collection::Dragon => artifact::create_dragon(&root, title)?,
        Collection::Idea => artifact::create_idea(&root, title)?,
        Collection::Sprint => artifact::create_sprint(&root, title)?,
        Collection::Task => artifact::create_task(&root, title)?,
    };
    let reachability = match collection {
        Collection::Dragon => artifact::probe_reachability(&root, &read::DRAGON, &created),
        Collection::Idea => artifact::probe_reachability(&root, &read::IDEA, &created),
        Collection::Sprint | Collection::Task => artifact::Reachability::Reachable,
    };
    if json {
        println!("{}", to_json(&created.record()));
    } else {
        println!(
            "created {} at {}",
            created.reference(),
            created.relative_path.display()
        );
    }
    if let artifact::Reachability::Degraded { blocker } = &reachability {
        eprintln!(
            "warning[degraded-repository]: created {} at {}, but repository \
             degradation currently blocks normal access — {blocker}; the \
             artifact was created and the exit status remains success; \
             repairing the blocker restores normal access",
            created.reference(),
            created.relative_path.display()
        );
    }
    Ok(())
}

/// List a collection's artifacts and render the requested projection.
///
/// `--active` narrows tasks to the active sprint's; it is meaningless for
/// other collections and refused rather than ignored.
fn list(collection: Collection, json: bool, active: bool) -> Result<(), Error> {
    let root = repo::discover(&cwd()?)?;
    let mut artifacts = scan(&root, collection)?;
    if active {
        if collection != Collection::Task {
            return Err(Error::InvalidInvocation {
                message: format!(
                    "`--active` filters tasks by the active sprint; it does \
                     not apply to `strata list {}s`",
                    collection.name()
                ),
            });
        }
        let sprints = read::scan_sprints(&root)?;
        let active_id = sprints
            .iter()
            .find(|sprint| sprint.summary.status == Status::Active)
            .map(|sprint| sprint.summary.id.clone());
        artifacts.retain(|task| task.summary.sprint == active_id);
    }
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
    // A sibling that blocks the strict scan gets the requested target
    // attached (decision 13): the diagnostic names what could not be
    // delivered as well as what blocked it.
    let display = target.to_string();
    let artifacts = match target {
        ArtifactTarget::Reference(reference) => {
            scan(&root, reference.collection).map_err(|err| err.blocking(&display))?
        }
        ArtifactTarget::Id(_) => {
            let union = || -> Result<Vec<read::Artifact>, Error> {
                let mut all = read::scan(&root, &read::DRAGON)?;
                all.extend(read::scan(&root, &read::IDEA)?);
                all.extend(read::scan_sprints(&root)?);
                all.extend(read::scan_tasks(&root)?);
                Ok(all)
            };
            union().map_err(|err| err.blocking(&display))?
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
