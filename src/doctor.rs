//! Repository validation: collect every problem instead of stopping at the
//! first.
//!
//! Ordinary reads abort on the first malformed file because their job is to
//! trust artifacts; `doctor`'s job is diagnosis, so it walks the same
//! per-file pipeline as [`crate::read`], converts each failure into a
//! [`Finding`], and adds repository-wide checks no single-file parse can
//! see: duplicate stable identities and duplicate display sequences.
//!
//! Validation never mutates canonical files. Per decision 5, states Git
//! inevitably produces are healthy: a missing managed directory is an empty
//! collection, not a finding.
//!
//! # Finding vocabulary
//!
//! `problem` codes are a provisional vocabulary (see task 0005): they are
//! deliberately few, aligned with the error categories where one applies,
//! and expected to be revisited when doctor covers more collections.
//!
//! Findings carry a [`Severity`] per decision 10's tiers: `error` findings
//! are corruption and make the repository unhealthy; `advice` findings are
//! repairable states (an unbound sugar edge, a lifecycle-contradicting
//! edge) that are reported without failing validation.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;

use crate::error::Error;
use crate::read::{self, Artifact, Collection};

/// How strongly one finding indicts the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Corruption: the repository is unhealthy until repaired.
    Error,
    /// Repairable: legal but weak state, reported without failing.
    Advice,
}

/// One validation finding. Serialized field names and order are a
/// compatibility surface pinned by tests; `path` is repository-relative
/// with `/` separators.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Provisional kebab-case problem code: `malformed-artifact`,
    /// `unreadable-artifact`, `artifact-conflict`, `duplicate-id`,
    /// `duplicate-sequence`, or a typed-edge code (`invalid-edge`,
    /// `dangling-edge`, `unbound-edge`, `stale-edge`).
    pub problem: &'static str,
    /// Repository-relative path of the affected file or directory.
    pub path: String,
    /// Human-oriented description; free to change.
    pub detail: String,
    /// Whether this finding is corruption or repairable advice.
    pub severity: Severity,
}

/// The outcome of one validation pass.
#[derive(Debug)]
pub struct Report {
    /// Every finding, sorted by path, then problem, then detail.
    pub findings: Vec<Finding>,
    /// Artifacts that parsed cleanly and entered the duplicate checks.
    pub artifacts_checked: usize,
}

impl Report {
    /// True when validation found no corruption; advice findings do not
    /// make a repository unhealthy.
    pub fn healthy(&self) -> bool {
        self.problems() == 0
    }

    /// Number of error-severity findings.
    pub fn problems(&self) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.severity == Severity::Error)
            .count()
    }
}

/// Validate the repository at `root` and report every problem found.
///
/// Returns `Err` only for environmental failures that prevent diagnosis
/// (an unreadable managed directory, a failing directory walk); problems
/// *in* the repository are findings, never errors.
pub fn check(root: &Path) -> Result<Report, Error> {
    let mut findings = Vec::new();
    let mut artifacts = Vec::new();

    for collection in [&read::DRAGON, &read::IDEA] {
        scan_dir(root, collection, &mut findings, &mut artifacts)?;
    }
    scan_sprints_dir(root, &mut findings, &mut artifacts)?;
    scan_task_dirs(root, &mut findings, &mut artifacts)?;

    let artifacts_checked = artifacts.len();
    findings.extend(duplicate_findings(&artifacts));

    // A task's `sprint:` field must name an existing sprint whose
    // containment directory holds the file (decision 11).
    let sprints: Vec<&Artifact> = artifacts
        .iter()
        .filter(|artifact| artifact.summary.kind == "sprint")
        .collect();
    let mut task_findings = Vec::new();
    for task in artifacts
        .iter()
        .filter(|artifact| artifact.summary.kind == "task")
    {
        let Some(sprint_id) = task.summary.sprint.as_deref() else {
            continue;
        };
        let Some(owner) = sprints.iter().find(|sprint| sprint.summary.id == sprint_id) else {
            task_findings.push(Finding {
                problem: "misfiled-task",
                path: task.summary.path.clone(),
                detail: format!(
                    "the `sprint:` field names `{sprint_id}`, but no sprint \
                     carries that id"
                ),
                severity: Severity::Error,
            });
            continue;
        };
        let owner_dir = owner
            .summary
            .path
            .rsplit_once('/')
            .map(|(dir, _)| dir)
            .unwrap_or_default();
        if task.summary.path.rsplit_once('/').map(|(dir, _)| dir) != Some(owner_dir) {
            task_findings.push(Finding {
                problem: "misfiled-task",
                path: task.summary.path.clone(),
                detail: format!(
                    "the `sprint:` field names `{sprint_id}` ({}), but the \
                     file sits outside that sprint's containment directory \
                     `{owner_dir}`",
                    owner.summary.reference()
                ),
                severity: Severity::Error,
            });
        }
    }
    findings.extend(task_findings);

    // At most one sprint may be active (the `new sprint` refusal, verified
    // here because a branch merge can produce the state no command allows).
    let active: Vec<&str> = artifacts
        .iter()
        .filter(|artifact| {
            artifact.summary.kind == "sprint" && artifact.summary.status == read::Status::Active
        })
        .map(|artifact| artifact.summary.path.as_str())
        .collect();
    if active.len() > 1 {
        findings.push(Finding {
            problem: "multiple-active-sprints",
            path: active[0].into(),
            detail: format!(
                "at most one sprint may be active, but {} are: {}",
                active.len(),
                active.join(", ")
            ),
            severity: Severity::Error,
        });
    }

    // Typed edges (decision 10): validated over the cleanly parsed
    // artifacts against every front-matter id in the archaeology tree, so
    // provenance targets in not-yet-managed collections still resolve.
    let universe = crate::edges::harvest_ids(root);
    for artifact in &artifacts {
        for edge_issue in
            crate::edges::check_artifact(&artifact.summary, &artifact.content, &universe)
        {
            findings.push(Finding {
                problem: edge_issue.problem,
                path: artifact.summary.path.clone(),
                detail: edge_issue.detail,
                severity: edge_issue.severity,
            });
        }
    }

    findings.sort_by(|a, b| (&a.path, a.problem, &a.detail).cmp(&(&b.path, b.problem, &b.detail)));
    Ok(Report {
        findings,
        artifacts_checked,
    })
}

/// Walk one collection directory, collecting per-file findings and cleanly
/// parsed artifacts.
fn scan_dir(
    root: &Path,
    collection: &Collection,
    findings: &mut Vec<Finding>,
    artifacts: &mut Vec<Artifact>,
) -> Result<(), Error> {
    let dir_rel = collection.dir;
    let dir = root.join(dir_rel);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        // A missing managed directory is an empty collection (decision 5):
        // Git does not round-trip empty directories.
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotADirectory => {
            findings.push(Finding {
                problem: "artifact-conflict",
                path: dir_rel.into(),
                detail: "a non-directory object occupies this managed directory path; \
                         move it aside"
                    .into(),
                severity: Severity::Error,
            });
            return Ok(());
        }
        Err(source) => {
            return Err(Error::Filesystem {
                operation: "read directory".into(),
                path: dir,
                source,
            });
        }
    };

    for entry in entries {
        let entry = entry.map_err(|source| Error::Filesystem {
            operation: "read directory entry".into(),
            path: dir.clone(),
            source,
        })?;
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            findings.push(Finding {
                problem: "malformed-artifact",
                path: format!("{dir_rel}/{}", name.to_string_lossy()),
                detail: "filename is not valid UTF-8".into(),
                severity: Severity::Error,
            });
            continue;
        };
        if name_str.starts_with('.') {
            continue;
        }
        let path = dir.join(name_str);
        if path.is_dir() {
            findings.push(Finding {
                problem: "artifact-conflict",
                path: format!("{dir_rel}/{name_str}"),
                detail: "a directory sits inside a managed collection \
                         directory; placement is flat (decision 11), so \
                         artifacts file directly in the collection directory"
                    .into(),
                severity: Severity::Error,
            });
            continue;
        }
        match read::parse_artifact(&path, dir_rel, name_str, collection) {
            Ok(artifact) => artifacts.push(artifact),
            Err(error) => findings.push(file_finding(error, dir_rel, name_str)),
        }
    }
    Ok(())
}

/// Walk the sprints directory, collecting per-sprint findings and cleanly
/// parsed sprint artifacts. Containment directories that fail structural
/// expectations (a loose file, a malformed `NNNN-slug` name, a missing
/// `sprint.md`) are findings; task files inside sprint directories are
/// validated separately.
fn scan_sprints_dir(
    root: &Path,
    findings: &mut Vec<Finding>,
    artifacts: &mut Vec<Artifact>,
) -> Result<(), Error> {
    let dir_rel = crate::repo::SPRINTS_DIR;
    let dir = root.join(dir_rel);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotADirectory => {
            findings.push(Finding {
                problem: "artifact-conflict",
                path: dir_rel.into(),
                detail: "a non-directory object occupies this managed directory path; \
                         move it aside"
                    .into(),
                severity: Severity::Error,
            });
            return Ok(());
        }
        Err(source) => {
            return Err(Error::Filesystem {
                operation: "read directory".into(),
                path: dir,
                source,
            });
        }
    };

    for entry in entries {
        let entry = entry.map_err(|source| Error::Filesystem {
            operation: "read directory entry".into(),
            path: dir.clone(),
            source,
        })?;
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            findings.push(Finding {
                problem: "malformed-artifact",
                path: format!("{dir_rel}/{}", name.to_string_lossy()),
                detail: "directory name is not valid UTF-8".into(),
                severity: Severity::Error,
            });
            continue;
        };
        if name_str.starts_with('.') {
            continue;
        }
        let sprint_dir = dir.join(name_str);
        let sprint_dir_rel = format!("{dir_rel}/{name_str}");
        if !sprint_dir.is_dir() {
            findings.push(Finding {
                problem: "malformed-artifact",
                path: sprint_dir_rel,
                detail: "the sprints directory holds one containment directory per \
                         sprint; a loose file cannot be a sprint artifact"
                    .into(),
                severity: Severity::Error,
            });
            continue;
        }
        let Some(sequence) = crate::artifact::parse_dir_sequence(name_str) else {
            findings.push(Finding {
                problem: "malformed-artifact",
                path: sprint_dir_rel,
                detail: "sprint containment directories must be named `NNNN-slug` \
                         with a four-digit display sequence"
                    .into(),
                severity: Severity::Error,
            });
            continue;
        };
        let file = sprint_dir.join(crate::repo::SPRINT_FILE);
        let file_rel = format!("{sprint_dir_rel}/{}", crate::repo::SPRINT_FILE);
        if !file.is_file() {
            findings.push(Finding {
                problem: "malformed-artifact",
                path: sprint_dir_rel,
                detail: format!(
                    "sprint containment directories must hold a `{}` artifact",
                    crate::repo::SPRINT_FILE
                ),
                severity: Severity::Error,
            });
            continue;
        }
        match read::parse_artifact_at(
            &file,
            &file_rel,
            sequence,
            "the containment directory name",
            &read::SPRINT,
        ) {
            Ok(artifact) => artifacts.push(artifact),
            Err(error) => findings.push(finding_at(error, file_rel)),
        }
    }
    Ok(())
}

/// Walk every sprint containment directory for task files, collecting
/// per-file findings and cleanly parsed task artifacts. Structural
/// problems with the containment directories themselves are
/// [`scan_sprints_dir`]'s findings, reported once there, so this walk
/// silently skips entries that are not well-formed task locations.
fn scan_task_dirs(
    root: &Path,
    findings: &mut Vec<Finding>,
    artifacts: &mut Vec<Artifact>,
) -> Result<(), Error> {
    let sprints_dir = root.join(crate::repo::SPRINTS_DIR);
    let entries = match fs::read_dir(&sprints_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry.map_err(|source| Error::Filesystem {
            operation: "read directory entry".into(),
            path: sprints_dir.clone(),
            source,
        })?;
        let name = entry.file_name();
        let Some(dir_name) = name.to_str() else {
            continue;
        };
        if dir_name.starts_with('.')
            || !entry.path().is_dir()
            || crate::artifact::parse_dir_sequence(dir_name).is_none()
        {
            continue;
        }
        let dir_rel = format!("{}/{dir_name}", crate::repo::SPRINTS_DIR);
        let dir = entry.path();
        let task_entries = match fs::read_dir(&dir) {
            Ok(task_entries) => task_entries,
            Err(_) => continue,
        };
        for task_entry in task_entries {
            let task_entry = task_entry.map_err(|source| Error::Filesystem {
                operation: "read directory entry".into(),
                path: dir.clone(),
                source,
            })?;
            let task_name = task_entry.file_name();
            let Some(task_name) = task_name.to_str() else {
                findings.push(Finding {
                    problem: "malformed-artifact",
                    path: format!("{dir_rel}/{}", task_entry.file_name().to_string_lossy()),
                    detail: "filename is not valid UTF-8".into(),
                    severity: Severity::Error,
                });
                continue;
            };
            if task_name.starts_with('.') || task_name == crate::repo::SPRINT_FILE {
                continue;
            }
            let path = dir.join(task_name);
            if path.is_dir() {
                findings.push(Finding {
                    problem: "artifact-conflict",
                    path: format!("{dir_rel}/{task_name}"),
                    detail: "a directory sits inside a sprint containment \
                             directory; tasks file directly in their sprint's \
                             directory (decision 11)"
                        .into(),
                    severity: Severity::Error,
                });
                continue;
            }
            match read::parse_artifact(&path, &dir_rel, task_name, &read::TASK) {
                Ok(artifact) => artifacts.push(artifact),
                Err(error) => findings.push(file_finding(error, &dir_rel, task_name)),
            }
        }
    }
    Ok(())
}

/// Convert one per-file parse failure into a finding.
fn file_finding(error: Error, dir_rel: &str, name: &str) -> Finding {
    finding_at(error, format!("{dir_rel}/{name}"))
}

/// Convert one per-file parse failure into a finding at `path`.
fn finding_at(error: Error, path: String) -> Finding {
    match error {
        Error::MalformedArtifact { reason, .. } => Finding {
            problem: "malformed-artifact",
            path,
            detail: reason,
            severity: Severity::Error,
        },
        Error::Filesystem {
            operation, source, ..
        } => Finding {
            problem: "unreadable-artifact",
            path,
            detail: format!("{operation} failed: {source}"),
            severity: Severity::Error,
        },
        // parse_dragon produces only the two variants above; anything else
        // would be a validation semantic doctor does not know how to
        // classify, which must fail loudly rather than pass silently.
        other => Finding {
            problem: "malformed-artifact",
            path,
            detail: other.to_string(),
            severity: Severity::Error,
        },
    }
}

/// Repository-wide duplicate checks over the cleanly parsed artifacts:
/// one finding per duplicated stable identity and per duplicated display
/// sequence, anchored at the first involved path and naming every other.
/// Identities are global — a stable id must be unique across collections —
/// while display sequences are collection-scoped, so `dragon:1` and
/// `idea:1` coexist.
fn duplicate_findings(artifacts: &[Artifact]) -> Vec<Finding> {
    let mut by_id: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut by_sequence: BTreeMap<(&str, u32), Vec<&str>> = BTreeMap::new();
    for artifact in artifacts {
        let summary = &artifact.summary;
        by_id.entry(&summary.id).or_default().push(&summary.path);
        by_sequence
            .entry((&summary.kind, summary.sequence))
            .or_default()
            .push(&summary.path);
    }

    let mut findings = Vec::new();
    for (id, mut paths) in by_id {
        if paths.len() > 1 {
            paths.sort_unstable();
            findings.push(Finding {
                problem: "duplicate-id",
                path: paths[0].into(),
                detail: format!("stable id `{id}` is shared by: {}", paths.join(", ")),
                severity: Severity::Error,
            });
        }
    }
    for ((kind, sequence), mut paths) in by_sequence {
        if paths.len() > 1 {
            paths.sort_unstable();
            findings.push(Finding {
                problem: "duplicate-sequence",
                path: paths[0].into(),
                detail: format!(
                    "display sequence {kind}:{sequence} is shared by: {}",
                    paths.join(", ")
                ),
                severity: Severity::Error,
            });
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo;
    use crate::repo::{DRAGONS_DIR, IDEAS_DIR};

    fn temp_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create temporary directory");
        repo::init(tmp.path()).expect("initialize repository");
        tmp
    }

    fn dragon_markdown(id: &str, sequence: u32, status: &str, title: &str) -> String {
        format!(
            "---\nid: {id}\nsequence: {sequence}\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# {title}\n"
        )
    }

    fn write_dragon(root: &Path, dir: &str, name: &str, content: &str) {
        fs::write(root.join(dir).join(name), content).unwrap();
    }

    fn problems(report: &Report) -> Vec<(&'static str, &str)> {
        report
            .findings
            .iter()
            .map(|f| (f.problem, f.path.as_str()))
            .collect()
    }

    #[test]
    fn healthy_repository_reports_no_findings() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-fine.md",
            &dragon_markdown("id-1", 1, "open", "Fine"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0002-done.md",
            &dragon_markdown("id-2", 2, "closed", "Done"),
        );

        let report = check(tmp.path()).unwrap();

        assert!(report.healthy(), "{:?}", report.findings);
        assert_eq!(report.artifacts_checked, 2);
    }

    #[test]
    fn marker_only_repository_is_healthy() {
        // The Git round-trip state from decision 5: marker without layout.
        let tmp = temp_repo();
        fs::remove_dir_all(tmp.path().join("archaeology")).unwrap();

        let report = check(tmp.path()).unwrap();

        assert!(report.healthy(), "{:?}", report.findings);
        assert_eq!(report.artifacts_checked, 0);
    }

    #[test]
    fn one_malformed_file_does_not_hide_the_rest() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-bad.md",
            "# No front matter\n",
        );
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0002-unknown-status.md",
            &dragon_markdown("id-2", 2, "resolved", "Unknown status"),
        );
        write_dragon(tmp.path(), DRAGONS_DIR, "junk.txt", "junk");

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![
                ("malformed-artifact", "archaeology/dragons/0001-bad.md"),
                (
                    "malformed-artifact",
                    "archaeology/dragons/0002-unknown-status.md"
                ),
                ("malformed-artifact", "archaeology/dragons/junk.txt"),
            ]
        );
    }

    #[test]
    fn duplicate_sequences_and_ids_are_single_findings_naming_every_path() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-a.md",
            &dragon_markdown("id-same", 1, "open", "A"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-b.md",
            &dragon_markdown("id-same", 1, "closed", "B"),
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(report.findings.len(), 2, "{:?}", report.findings);
        let by_problem: Vec<&str> = report.findings.iter().map(|f| f.problem).collect();
        assert!(by_problem.contains(&"duplicate-id"));
        assert!(by_problem.contains(&"duplicate-sequence"));
        for finding in &report.findings {
            assert_eq!(finding.path, "archaeology/dragons/0001-a.md");
            assert!(
                finding.detail.contains("archaeology/dragons/0001-a.md")
                    && finding.detail.contains("archaeology/dragons/0001-b.md"),
                "detail must name every involved path: {}",
                finding.detail
            );
        }
    }

    fn idea_markdown(id: &str, sequence: u32, status: &str, title: &str) -> String {
        format!(
            "---\nid: {id}\nsequence: {sequence}\nkind: idea\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# {title}\n"
        )
    }

    fn seed_idea(root: &Path, dir: &str, name: &str, content: &str) {
        fs::create_dir_all(root.join(dir)).unwrap();
        fs::write(root.join(dir).join(name), content).unwrap();
    }

    #[test]
    fn ideas_are_validated_alongside_dragons() {
        let tmp = temp_repo();
        seed_idea(
            tmp.path(),
            IDEAS_DIR,
            "0001-fine.md",
            &idea_markdown("idea-fine", 1, "parked", "Fine"),
        );
        seed_idea(
            tmp.path(),
            IDEAS_DIR,
            "0002-settled.md",
            &idea_markdown("idea-settled", 2, "adopted", "Settled"),
        );
        // A dragon status is not an idea status.
        seed_idea(
            tmp.path(),
            IDEAS_DIR,
            "0003-wrong-status.md",
            &idea_markdown("idea-wrong", 3, "open", "Wrong status"),
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(report.artifacts_checked, 2);
        assert_eq!(
            problems(&report),
            vec![(
                "malformed-artifact",
                "archaeology/ideas/0003-wrong-status.md"
            )]
        );
    }

    #[test]
    fn display_sequences_are_collection_scoped_but_ids_are_global() {
        let tmp = temp_repo();
        // dragon:1 and idea:1 legitimately coexist.
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-risk.md",
            &dragon_markdown("shared-id", 1, "open", "Risk"),
        );
        // The same stable id across collections is corruption.
        seed_idea(
            tmp.path(),
            IDEAS_DIR,
            "0001-idea.md",
            &idea_markdown("shared-id", 1, "parked", "Idea"),
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![("duplicate-id", "archaeology/dragons/0001-risk.md")]
        );
        assert!(
            report.findings[0].detail.contains("shared-id"),
            "{}",
            report.findings[0].detail
        );
    }

    /// Seed an unmanaged decision artifact so the id universe contains a
    /// legal typed-edge target.
    fn seed_decision(root: &Path, id: &str) {
        let dir = root.join("archaeology/decisions");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("0001-a-decision.md"),
            format!(
                "---\nid: {id}\nsequence: 1\nkind: decision\nstatus: accepted\ncreated: 2026-07-20\n---\n\n# A decision\n"
            ),
        )
        .unwrap();
    }

    fn closed_dragon_with_edge(edge_line: &str) -> String {
        format!(
            "---\nid: drg-settled\nsequence: 1\nkind: dragon\nstatus: closed\ncreated: 2026-07-20\n{edge_line}\n---\n\n# Settled\n"
        )
    }

    #[test]
    fn valid_provenance_edges_pass_and_target_unmanaged_artifacts() {
        let tmp = temp_repo();
        seed_decision(tmp.path(), "dec-settles-it");
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-settled.md",
            &closed_dragon_with_edge("resolved-by: \"[[dec-settles-it|the settling decision]]\""),
        );

        let report = check(tmp.path()).unwrap();

        assert!(report.healthy(), "{:?}", report.findings);
        assert!(report.findings.is_empty(), "{:?}", report.findings);
    }

    #[test]
    fn dangling_edge_target_is_an_error() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-settled.md",
            &closed_dragon_with_edge("resolved-by: \"[[dec-nowhere|gone]]\""),
        );

        let report = check(tmp.path()).unwrap();

        assert!(!report.healthy());
        assert_eq!(
            problems(&report),
            vec![("dangling-edge", "archaeology/dragons/0001-settled.md")]
        );
        assert!(report.findings[0].detail.contains("dec-nowhere"));
    }

    #[test]
    fn edge_targeting_an_idea_is_an_error() {
        let tmp = temp_repo();
        seed_idea(
            tmp.path(),
            IDEAS_DIR,
            "0001-idea.md",
            &idea_markdown("idea-tempting", 1, "parked", "Tempting"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-settled.md",
            &closed_dragon_with_edge("resolved-by: \"[[idea-tempting|an idea]]\""),
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![("invalid-edge", "archaeology/dragons/0001-settled.md")]
        );
        assert!(
            report.findings[0].detail.contains("idea"),
            "{}",
            report.findings[0].detail
        );
    }

    #[test]
    fn sugar_edge_value_is_advice_not_corruption() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-settled.md",
            &closed_dragon_with_edge("resolved-by: \"[[decision:1]]\""),
        );

        let report = check(tmp.path()).unwrap();

        assert!(report.healthy(), "sugar must not fail validation");
        assert_eq!(
            problems(&report),
            vec![("unbound-edge", "archaeology/dragons/0001-settled.md")]
        );
        assert_eq!(report.findings[0].severity, Severity::Advice);
    }

    #[test]
    fn lifecycle_contradicting_edge_is_advice() {
        let tmp = temp_repo();
        seed_decision(tmp.path(), "dec-settles-it");
        // A reopened dragon still carrying its resolution edge.
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-reopened.md",
            "---\nid: drg-reopened\nsequence: 1\nkind: dragon\nstatus: open\ncreated: 2026-07-20\nresolved-by: \"[[dec-settles-it|stale claim]]\"\n---\n\n# Reopened\n",
        );

        let report = check(tmp.path()).unwrap();

        assert!(report.healthy());
        assert_eq!(
            problems(&report),
            vec![("stale-edge", "archaeology/dragons/0001-reopened.md")]
        );
    }

    #[test]
    fn edge_on_the_wrong_source_kind_is_an_error() {
        let tmp = temp_repo();
        seed_idea(
            tmp.path(),
            IDEAS_DIR,
            "0001-idea.md",
            "---\nid: idea-confused\nsequence: 1\nkind: idea\nstatus: parked\ncreated: 2026-07-20\nresolved-by: \"[[dec-x|label]]\"\n---\n\n# Confused\n",
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![("invalid-edge", "archaeology/ideas/0001-idea.md")]
        );
        assert!(
            report.findings[0].detail.contains("belong on dragons"),
            "{}",
            report.findings[0].detail
        );
    }

    #[test]
    fn unquoted_edge_marker_is_an_error_naming_the_yaml_footgun() {
        let tmp = temp_repo();
        seed_decision(tmp.path(), "dec-settles-it");
        // Without quotes, YAML parses `[[a|b]]` as a nested flow sequence.
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-settled.md",
            &closed_dragon_with_edge("resolved-by: [[dec-settles-it|label]]"),
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![("invalid-edge", "archaeology/dragons/0001-settled.md")]
        );
        assert!(
            report.findings[0].detail.contains("quoted"),
            "{}",
            report.findings[0].detail
        );
    }

    #[test]
    fn unknown_front_matter_keys_are_inert() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-settled.md",
            &closed_dragon_with_edge("supersedes: \"[[dec-nowhere|not vocabulary]]\""),
        );

        let report = check(tmp.path()).unwrap();

        assert!(
            report.findings.is_empty(),
            "keys outside the vocabulary are data: {:?}",
            report.findings
        );
    }

    #[test]
    fn non_directory_at_managed_path_is_a_conflict_finding() {
        let tmp = temp_repo();
        fs::remove_dir(tmp.path().join(DRAGONS_DIR)).unwrap();
        fs::write(tmp.path().join(DRAGONS_DIR), "not a directory").unwrap();

        let report = check(tmp.path()).unwrap();

        assert_eq!(problems(&report), vec![("artifact-conflict", DRAGONS_DIR)]);
    }

    #[test]
    fn leftover_lifecycle_subdirectory_is_a_conflict_finding() {
        let tmp = temp_repo();
        fs::create_dir(tmp.path().join(DRAGONS_DIR).join("open")).unwrap();

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![("artifact-conflict", "archaeology/dragons/open")]
        );
        assert!(
            report.findings[0].detail.contains("flat"),
            "{}",
            report.findings[0].detail
        );
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_file_is_reported_not_fatal() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_DIR,
            "0001-fine.md",
            &dragon_markdown("id-1", 1, "open", "Fine"),
        );
        let locked = tmp.path().join(DRAGONS_DIR).join("0002-locked.md");
        fs::write(&locked, "locked").unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        let report = check(tmp.path()).unwrap();

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(report.artifacts_checked, 1);
        assert_eq!(
            problems(&report),
            vec![("unreadable-artifact", "archaeology/dragons/0002-locked.md")]
        );
    }

    #[test]
    fn findings_are_sorted_deterministically_by_path() {
        let tmp = temp_repo();
        write_dragon(tmp.path(), DRAGONS_DIR, "0002-b.md", "no front matter");
        write_dragon(tmp.path(), DRAGONS_DIR, "0001-a.md", "no front matter");
        write_dragon(tmp.path(), DRAGONS_DIR, "0003-c.md", "no front matter");

        let paths: Vec<String> = check(tmp.path())
            .unwrap()
            .findings
            .into_iter()
            .map(|f| f.path)
            .collect();

        assert_eq!(
            paths,
            vec![
                "archaeology/dragons/0001-a.md",
                "archaeology/dragons/0002-b.md",
                "archaeology/dragons/0003-c.md",
            ]
        );
    }

    fn seed_sprint(root: &Path, dir_name: &str, sequence: u32, status: &str) {
        let dir = root.join(crate::repo::SPRINTS_DIR).join(dir_name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(crate::repo::SPRINT_FILE),
            format!(
                "---\nid: spr-{sequence}\nsequence: {sequence}\nkind: sprint\nstatus: {status}\ncreated: 2026-07-20\n---\n\n# Sprint {sequence}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn sprints_are_validated_and_a_single_active_sprint_is_healthy() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0001-history", 1, "closed");
        seed_sprint(tmp.path(), "0002-current", 2, "active");

        let report = check(tmp.path()).unwrap();

        assert!(report.healthy(), "{:?}", report.findings);
        assert_eq!(report.artifacts_checked, 2);
    }

    #[test]
    fn multiple_active_sprints_are_an_error_naming_every_path() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0001-branch-a", 1, "active");
        seed_sprint(tmp.path(), "0002-branch-b", 2, "active");

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![(
                "multiple-active-sprints",
                "archaeology/sprints/0001-branch-a/sprint.md"
            )]
        );
        assert!(
            report.findings[0].detail.contains("0002-branch-b"),
            "{}",
            report.findings[0].detail
        );
    }

    #[test]
    fn malformed_sprint_directories_are_findings_not_fatal() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0001-fine", 1, "closed");
        fs::create_dir_all(tmp.path().join(crate::repo::SPRINTS_DIR).join("0002-empty")).unwrap();
        fs::write(
            tmp.path().join(crate::repo::SPRINTS_DIR).join("loose.md"),
            "junk",
        )
        .unwrap();

        let report = check(tmp.path()).unwrap();

        assert_eq!(report.artifacts_checked, 1);
        assert_eq!(
            problems(&report),
            vec![
                ("malformed-artifact", "archaeology/sprints/0002-empty"),
                ("malformed-artifact", "archaeology/sprints/loose.md"),
            ]
        );
    }

    #[test]
    fn tasks_misfiled_outside_their_named_sprint_are_errors() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0001-a", 1, "closed");
        seed_sprint(tmp.path(), "0002-b", 2, "active");
        // The task names sprint 2 but sits in sprint 1's directory.
        fs::write(
            tmp.path()
                .join(crate::repo::SPRINTS_DIR)
                .join("0001-a")
                .join("0001-wandering.md"),
            "---\nid: tsk-wandering\nsequence: 1\nkind: task\nstatus: closed\nsprint: spr-2\ncreated: 2026-07-20\n---\n\n# Wandering\n",
        )
        .unwrap();

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![(
                "misfiled-task",
                "archaeology/sprints/0001-a/0001-wandering.md"
            )]
        );
        assert!(
            report.findings[0].detail.contains("0002-b"),
            "{}",
            report.findings[0].detail
        );
    }

    #[test]
    fn finding_json_field_names_and_order_are_stable() {
        let finding = Finding {
            problem: "duplicate-sequence",
            path: "archaeology/dragons/0001-a.md".into(),
            detail: "display sequence 1 is shared by: a, b".into(),
            severity: Severity::Error,
        };

        assert_eq!(
            serde_json::to_string(&finding).unwrap(),
            "{\"problem\":\"duplicate-sequence\",\
             \"path\":\"archaeology/dragons/0001-a.md\",\
             \"detail\":\"display sequence 1 is shared by: a, b\",\
             \"severity\":\"error\"}"
        );
    }
}
