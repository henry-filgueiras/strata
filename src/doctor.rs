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

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;

use crate::error::Error;
use crate::read::{self, Artifact, Collection, Status};

/// One validation problem. Serialized field names and order are a
/// compatibility surface pinned by tests; `path` is repository-relative
/// with `/` separators.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Provisional kebab-case problem code: `malformed-artifact`,
    /// `unreadable-artifact`, `artifact-conflict`, `duplicate-id`, or
    /// `duplicate-sequence`.
    pub problem: &'static str,
    /// Repository-relative path of the affected file or directory.
    pub path: String,
    /// Human-oriented description; free to change.
    pub detail: String,
}

/// The outcome of one validation pass.
#[derive(Debug)]
pub struct Report {
    /// Every problem found, sorted by path, then problem, then detail.
    pub findings: Vec<Finding>,
    /// Artifacts that parsed cleanly and entered the duplicate checks.
    pub artifacts_checked: usize,
}

impl Report {
    /// True when validation found nothing wrong.
    pub fn healthy(&self) -> bool {
        self.findings.is_empty()
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
        for (status, dir_rel) in collection.states {
            scan_dir(
                root,
                collection,
                dir_rel,
                *status,
                &mut findings,
                &mut artifacts,
            )?;
        }
    }

    let artifacts_checked = artifacts.len();
    findings.extend(duplicate_findings(&artifacts));
    findings.sort_by(|a, b| (&a.path, a.problem, &a.detail).cmp(&(&b.path, b.problem, &b.detail)));
    Ok(Report {
        findings,
        artifacts_checked,
    })
}

/// Walk one managed directory, collecting per-file findings and cleanly
/// parsed artifacts.
fn scan_dir(
    root: &Path,
    collection: &Collection,
    dir_rel: &str,
    status: Status,
    findings: &mut Vec<Finding>,
    artifacts: &mut Vec<Artifact>,
) -> Result<(), Error> {
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
            });
            continue;
        };
        if name_str.starts_with('.') {
            continue;
        }
        match read::parse_artifact(&dir.join(name_str), dir_rel, name_str, collection, status) {
            Ok(artifact) => artifacts.push(artifact),
            Err(error) => findings.push(file_finding(error, dir_rel, name_str)),
        }
    }
    Ok(())
}

/// Convert one per-file parse failure into a finding.
fn file_finding(error: Error, dir_rel: &str, name: &str) -> Finding {
    let path = format!("{dir_rel}/{name}");
    match error {
        Error::MalformedArtifact { reason, .. } => Finding {
            problem: "malformed-artifact",
            path,
            detail: reason,
        },
        Error::Filesystem {
            operation, source, ..
        } => Finding {
            problem: "unreadable-artifact",
            path,
            detail: format!("{operation} failed: {source}"),
        },
        // parse_dragon produces only the two variants above; anything else
        // would be a validation semantic doctor does not know how to
        // classify, which must fail loudly rather than pass silently.
        other => Finding {
            problem: "malformed-artifact",
            path,
            detail: other.to_string(),
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
            });
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo;
    use crate::repo::{DRAGONS_CLOSED_DIR, DRAGONS_OPEN_DIR, IDEAS_ADOPTED_DIR, IDEAS_PARKED_DIR};

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
            DRAGONS_OPEN_DIR,
            "0001-fine.md",
            &dragon_markdown("id-1", 1, "open", "Fine"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_CLOSED_DIR,
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
            DRAGONS_OPEN_DIR,
            "0001-bad.md",
            "# No front matter\n",
        );
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0002-misplaced.md",
            &dragon_markdown("id-2", 2, "closed", "Misplaced"),
        );
        write_dragon(tmp.path(), DRAGONS_OPEN_DIR, "junk.txt", "junk");

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![
                ("malformed-artifact", "archaeology/dragons/open/0001-bad.md"),
                (
                    "malformed-artifact",
                    "archaeology/dragons/open/0002-misplaced.md"
                ),
                ("malformed-artifact", "archaeology/dragons/open/junk.txt"),
            ]
        );
    }

    #[test]
    fn duplicate_sequences_and_ids_are_single_findings_naming_every_path() {
        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-a.md",
            &dragon_markdown("id-same", 1, "open", "A"),
        );
        write_dragon(
            tmp.path(),
            DRAGONS_CLOSED_DIR,
            "0001-b.md",
            &dragon_markdown("id-same", 1, "closed", "B"),
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(report.findings.len(), 2, "{:?}", report.findings);
        let by_problem: Vec<&str> = report.findings.iter().map(|f| f.problem).collect();
        assert!(by_problem.contains(&"duplicate-id"));
        assert!(by_problem.contains(&"duplicate-sequence"));
        for finding in &report.findings {
            assert_eq!(finding.path, "archaeology/dragons/closed/0001-b.md");
            assert!(
                finding
                    .detail
                    .contains("archaeology/dragons/open/0001-a.md")
                    && finding
                        .detail
                        .contains("archaeology/dragons/closed/0001-b.md"),
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
            IDEAS_PARKED_DIR,
            "0001-fine.md",
            &idea_markdown("idea-fine", 1, "parked", "Fine"),
        );
        // Adopted status in the parked directory: lifecycle mismatch.
        seed_idea(
            tmp.path(),
            IDEAS_PARKED_DIR,
            "0002-misplaced.md",
            &idea_markdown("idea-misplaced", 2, "adopted", "Misplaced"),
        );
        // A dragon status is not an idea status.
        seed_idea(
            tmp.path(),
            IDEAS_ADOPTED_DIR,
            "0003-wrong-status.md",
            &idea_markdown("idea-wrong", 3, "open", "Wrong status"),
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(report.artifacts_checked, 1);
        assert_eq!(
            problems(&report),
            vec![
                (
                    "malformed-artifact",
                    "archaeology/ideas/adopted/0003-wrong-status.md"
                ),
                (
                    "malformed-artifact",
                    "archaeology/ideas/parked/0002-misplaced.md"
                ),
            ]
        );
    }

    #[test]
    fn display_sequences_are_collection_scoped_but_ids_are_global() {
        let tmp = temp_repo();
        // dragon:1 and idea:1 legitimately coexist.
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-risk.md",
            &dragon_markdown("shared-id", 1, "open", "Risk"),
        );
        // The same stable id across collections is corruption.
        seed_idea(
            tmp.path(),
            IDEAS_PARKED_DIR,
            "0001-idea.md",
            &idea_markdown("shared-id", 1, "parked", "Idea"),
        );

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![("duplicate-id", "archaeology/dragons/open/0001-risk.md")]
        );
        assert!(
            report.findings[0].detail.contains("shared-id"),
            "{}",
            report.findings[0].detail
        );
    }

    #[test]
    fn non_directory_at_managed_path_is_a_conflict_finding() {
        let tmp = temp_repo();
        fs::remove_dir(tmp.path().join(DRAGONS_CLOSED_DIR)).unwrap();
        fs::write(tmp.path().join(DRAGONS_CLOSED_DIR), "not a directory").unwrap();

        let report = check(tmp.path()).unwrap();

        assert_eq!(
            problems(&report),
            vec![("artifact-conflict", DRAGONS_CLOSED_DIR)]
        );
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_file_is_reported_not_fatal() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = temp_repo();
        write_dragon(
            tmp.path(),
            DRAGONS_OPEN_DIR,
            "0001-fine.md",
            &dragon_markdown("id-1", 1, "open", "Fine"),
        );
        let locked = tmp.path().join(DRAGONS_OPEN_DIR).join("0002-locked.md");
        fs::write(&locked, "locked").unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        let report = check(tmp.path()).unwrap();

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(report.artifacts_checked, 1);
        assert_eq!(
            problems(&report),
            vec![(
                "unreadable-artifact",
                "archaeology/dragons/open/0002-locked.md"
            )]
        );
    }

    #[test]
    fn findings_are_sorted_deterministically_by_path() {
        let tmp = temp_repo();
        write_dragon(tmp.path(), DRAGONS_OPEN_DIR, "0002-b.md", "no front matter");
        write_dragon(tmp.path(), DRAGONS_OPEN_DIR, "0001-a.md", "no front matter");
        write_dragon(
            tmp.path(),
            DRAGONS_CLOSED_DIR,
            "0003-c.md",
            "no front matter",
        );

        let paths: Vec<String> = check(tmp.path())
            .unwrap()
            .findings
            .into_iter()
            .map(|f| f.path)
            .collect();

        assert_eq!(
            paths,
            vec![
                "archaeology/dragons/closed/0003-c.md",
                "archaeology/dragons/open/0001-a.md",
                "archaeology/dragons/open/0002-b.md",
            ]
        );
    }

    #[test]
    fn finding_json_field_names_and_order_are_stable() {
        let finding = Finding {
            problem: "duplicate-sequence",
            path: "archaeology/dragons/open/0001-a.md".into(),
            detail: "display sequence 1 is shared by: a, b".into(),
        };

        assert_eq!(
            serde_json::to_string(&finding).unwrap(),
            "{\"problem\":\"duplicate-sequence\",\
             \"path\":\"archaeology/dragons/open/0001-a.md\",\
             \"detail\":\"display sequence 1 is shared by: a, b\"}"
        );
    }
}
