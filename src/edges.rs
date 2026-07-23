//! Reference markers and typed edge validation, per decision 10
//! (`dec-reference-syntax`).
//!
//! A reference marker is wikilink-style, one grammar with two strictness
//! levels: the canonical bound form `[[stable-id|label]]` and the unbound
//! sugar form `[[kind:N]]` (label optional). Typed edges are front-matter
//! fields whose values are bound markers as quoted YAML strings; the
//! vocabulary is a closed allowlist ([`EDGE_KINDS`]) grown only by
//! decision, and keys outside it are inert data.
//!
//! Validation implements the decision's three tiers: corruption
//! (unparseable values, dangling bound targets, forbidden target kinds),
//! repairable (sugar values, lifecycle-contradicting edges), and advisory
//! (absence — deliberately not checked here; that promotion is idea 13's
//! question).
//!
//! The verification universe is every front-matter `id` in the archaeology
//! tree, managed or not, so edges may target decisions and tasks before
//! those collections are managed. Identities resolve through the
//! repository-wide claimant [`Catalog`] (task 23, decision 12): every
//! admitted claim is retained with an explicit disposition, and an id is
//! classified as missing, unique, or ambiguous — no consumer selects
//! among ambiguous claimants.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::doctor::Severity;
use crate::read::{Status, Summary};

/// One parsed reference marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker<'a> {
    /// Canonical form: stable id plus mandatory frozen label.
    Bound { id: &'a str, label: &'a str },
    /// Sugar form: a `kind:N` reference, label optional. Legal but weak;
    /// repaired by an explicit bind.
    Sugar {
        reference: &'a str,
        label: Option<&'a str>,
    },
}

/// Parse one reference marker, or `None` when the text is not a marker.
///
/// Grammar per decision 10: `[[` target (`|` label)? `]]` on one line;
/// targets contain no whitespace, `|`, `#` (reserved for future
/// fragments), or `]]`; a target containing `:` is sugar, anything else is
/// a stable id; bound markers require a nonempty label, sugar may omit it.
pub fn parse_marker(text: &str) -> Option<Marker<'_>> {
    let inner = text.strip_prefix("[[")?.strip_suffix("]]")?;
    if inner.contains(']') || inner.contains('\n') {
        return None;
    }
    let (target, label) = match inner.split_once('|') {
        Some((target, label)) => (target, Some(label)),
        None => (inner, None),
    };
    if target.is_empty()
        || target.contains('#')
        || target.contains('|')
        || target.chars().any(char::is_whitespace)
    {
        return None;
    }
    if label.is_some_and(str::is_empty) {
        return None;
    }
    if target.contains(':') {
        Some(Marker::Sugar {
            reference: target,
            label,
        })
    } else {
        Some(Marker::Bound {
            id: target,
            label: label?,
        })
    }
}

/// One typed edge kind from the decided vocabulary.
#[derive(Debug)]
pub struct EdgeKind {
    /// Front-matter key.
    pub key: &'static str,
    /// The artifact kind this edge belongs on.
    pub source_kind: &'static str,
    /// The lifecycle state whose settlement the edge records; the edge on
    /// a source in any other state is a repairable contradiction.
    pub settled_status: Status,
    /// Artifact kinds a bound target may resolve to. Ideas are absent
    /// from every list: no typed edge may target one.
    pub target_kinds: &'static [&'static str],
}

/// The decided vocabulary: terminal-provenance edges only, each landed
/// with its first consumer.
pub const EDGE_KINDS: &[EdgeKind] = &[
    EdgeKind {
        key: "resolved-by",
        source_kind: "dragon",
        settled_status: Status::Closed,
        target_kinds: &["decision", "task"],
    },
    EdgeKind {
        key: "adopted-by",
        source_kind: "idea",
        settled_status: Status::Adopted,
        target_kinds: &["decision", "task"],
    },
];

/// One validation outcome for a typed edge, before doctor attaches the
/// artifact path.
#[derive(Debug)]
pub struct EdgeIssue {
    pub severity: Severity,
    pub problem: &'static str,
    pub detail: String,
}

fn issue(severity: Severity, problem: &'static str, detail: String) -> EdgeIssue {
    EdgeIssue {
        severity,
        problem,
        detail,
    }
}

/// Validate every typed edge on one parsed artifact against the
/// repository's identity claimant catalog.
pub(crate) fn check_artifact(
    summary: &Summary,
    content: &str,
    catalog: &Catalog,
) -> Vec<EdgeIssue> {
    let mut issues = Vec::new();
    let Some((front_matter, _)) = crate::read::split_front_matter(content) else {
        return issues;
    };
    let Ok(mapping) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(front_matter) else {
        return issues;
    };

    for kind in EDGE_KINDS {
        let Some(value) = mapping.get(kind.key) else {
            continue;
        };
        if summary.kind != kind.source_kind {
            issues.push(issue(
                Severity::Error,
                "invalid-edge",
                format!(
                    "`{}` typed edges belong on {}s, but this artifact is a {}",
                    kind.key, kind.source_kind, summary.kind
                ),
            ));
            continue;
        }
        let markers: Vec<&serde_yaml_ng::Value> = match value {
            serde_yaml_ng::Value::Sequence(sequence) => sequence.iter().collect(),
            other => vec![other],
        };
        for marker_value in markers {
            let Some(text) = marker_value.as_str() else {
                issues.push(issue(
                    Severity::Error,
                    "invalid-edge",
                    format!(
                        "`{}` must be a reference marker as a quoted YAML \
                         string (or a sequence of them); unquoted `[[...]]` \
                         parses as a YAML list, not a marker",
                        kind.key
                    ),
                ));
                continue;
            };
            issues.extend(check_marker(kind, summary, text, catalog));
        }
    }
    issues
}

/// Validate one marker string carried by a typed edge.
fn check_marker(
    kind: &EdgeKind,
    summary: &Summary,
    text: &str,
    catalog: &Catalog,
) -> Option<EdgeIssue> {
    match parse_marker(text) {
        None => Some(issue(
            Severity::Error,
            "invalid-edge",
            format!(
                "`{}` value `{text}` is not a reference marker; expected \
                 `[[stable-id|label]]`",
                kind.key
            ),
        )),
        Some(Marker::Sugar { reference, .. }) => Some(issue(
            Severity::Advice,
            "unbound-edge",
            format!(
                "`{}` holds the unbound sugar reference `{reference}`; bind \
                 it to a stable id so the edge is verifiable",
                kind.key
            ),
        )),
        Some(Marker::Bound { id, .. }) => match catalog.resolve(id) {
            Resolution::Missing => Some(issue(
                Severity::Error,
                "dangling-edge",
                format!(
                    "`{}` targets `{id}`, but no artifact in this repository \
                     carries that id",
                    kind.key
                ),
            )),
            Resolution::Ambiguous(claimants) => Some(issue(
                Severity::Error,
                "ambiguous-edge",
                format!(
                    "`{}` targets `{id}`, which {} artifacts claim: {}; an \
                     ambiguous identity cannot verify an edge — repair the \
                     duplicated ids",
                    kind.key,
                    claimants.len(),
                    claimant_paths(&claimants).join(", ")
                ),
            )),
            Resolution::Unique(claimant)
                if !kind.target_kinds.contains(&claimant.claim.kind.as_str()) =>
            {
                Some(issue(
                    Severity::Error,
                    "invalid-edge",
                    format!(
                        "`{}` targets `{id}`, a {}; legal targets \
                         are: {}",
                        kind.key,
                        claimant.claim.kind,
                        kind.target_kinds.join(", ")
                    ),
                ))
            }
            Resolution::Unique(_) if summary.status != kind.settled_status => Some(issue(
                Severity::Advice,
                "stale-edge",
                format!(
                    "`{}` claims settlement, but this {} is {}, not {}; \
                     investigate or remove the edge",
                    kind.key, summary.kind, summary.status, kind.settled_status
                ),
            )),
            Resolution::Unique(_) => None,
        },
    }
}

/// Repository-relative paths of `claimants`, preserving their path-sorted
/// order.
fn claimant_paths(claimants: &[&Claimant]) -> Vec<String> {
    claimants
        .iter()
        .map(|claimant| claimant.claim.path.clone())
        .collect()
}

/// One artifact harvested from the archaeology tree, managed or not:
/// enough identity to serve as a typed-edge target.
#[derive(Debug, Clone)]
pub(crate) struct Harvested {
    pub id: String,
    pub kind: String,
    pub sequence: Option<u32>,
    pub title: Option<String>,
    /// Repository-relative path with `/` separators, for error messages.
    pub path: String,
}

/// Best-effort harvest of every front-matter-bearing artifact in the
/// archaeology tree, managed or not.
///
/// Files without parseable front matter, non-Markdown files, and dot
/// entries are skipped silently: the harvest answers "what could this
/// reference resolve to", not "is this file valid". Traversal is sorted
/// so duplicates surface deterministically; duplicate ids among managed
/// artifacts are separately real findings.
///
/// The walk honors the task 22 filesystem boundary: every entry is
/// classified without following symlinks, so a symlinked directory is
/// never descended and a symlinked file is never read — an identity
/// reachable only through a link stays outside the verification universe,
/// and traversal cycles are impossible without canonicalization. Reads go
/// through the bounded [`crate::read::read_artifact_bytes`] seam;
/// oversized files are skipped like any other unharvestable content.
pub(crate) fn harvest(root: &Path) -> Vec<Harvested> {
    let mut harvested = Vec::new();
    let archaeology = root.join("archaeology");
    let is_real_dir = fs::symlink_metadata(&archaeology)
        .map(|meta| meta.is_dir())
        .unwrap_or(false);
    if !is_real_dir {
        return harvested;
    }
    let mut stack = vec![archaeology];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut paths: Vec<(PathBuf, fs::FileType)> = entries
            .flatten()
            .filter_map(|entry| {
                entry
                    .file_type()
                    .ok()
                    .map(|file_type| (entry.path(), file_type))
            })
            .collect();
        paths.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, file_type) in paths {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if !file_type.is_file() || !name.ends_with(".md") {
                continue;
            }
            let Ok(content) = crate::read::read_artifact_bytes(&path) else {
                continue;
            };
            let Some((front_matter, body)) = crate::read::split_front_matter(&content) else {
                continue;
            };
            let Ok(mapping) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(front_matter) else {
                continue;
            };
            if let (Some(id), Some(kind)) = (
                mapping.get("id").and_then(|v| v.as_str()),
                mapping.get("kind").and_then(|v| v.as_str()),
            ) {
                harvested.push(Harvested {
                    id: id.to_string(),
                    kind: kind.to_string(),
                    sequence: mapping
                        .get("sequence")
                        .and_then(|v| v.as_u64())
                        .and_then(|v| u32::try_from(v).ok()),
                    title: crate::read::extract_title(body).ok(),
                    path: path
                        .strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/"),
                });
            }
        }
    }
    harvested
}

/// Canonical-parse verdict for one admitted identity claimant, per
/// decision 12 (`dec-canonical-representation`): the disposition is
/// explicit catalog data. Callers must not infer validity from absent
/// optional fields such as `sequence` or `title`, and must not
/// reclassify parse failures ad hoc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Disposition {
    /// The claimant passes the canonical artifact parse of its managed
    /// position.
    Canonical,
    /// The claim was admitted, but no canonical-parse verdict exists for
    /// it: the file does not sit at a strongly managed artifact position
    /// (probe-only).
    Unassessed,
    /// The canonical parse refused the file; `class` is the stable
    /// problem class of the refusal, aligned with doctor's finding
    /// vocabulary.
    Rejected { class: &'static str },
}

/// One admitted identity claimant: the harvested claim plus its explicit
/// canonical-parse disposition.
#[derive(Debug, Clone)]
pub(crate) struct Claimant {
    pub claim: Harvested,
    pub disposition: Disposition,
}

/// The repository-wide identity claimant catalog (task 23, thread 5,
/// decision 12).
///
/// Built from the single [`harvest`] pass, the catalog retains every
/// admitted claim — canonical, probe-only, and rejected-by-canonical-
/// parsing claimants alike — ordered by repository-relative path,
/// independent of directory enumeration or file-creation order. Every
/// identity consumer classifies ids through [`Catalog::resolve`]; no
/// caller may collapse claimants first-seen-wins.
pub(crate) struct Catalog {
    /// Every admitted claimant, sorted by repository-relative path.
    claimants: Vec<Claimant>,
}

/// The identity-resolution algebra (thread 5's invariant): an id is
/// missing, unique, or ambiguous, and no command or doctor check silently
/// chooses among ambiguous claimants.
pub(crate) enum Resolution<'a> {
    /// No admitted claimant carries the id.
    Missing,
    /// Exactly one admitted claimant carries the id, whatever its
    /// disposition.
    Unique(&'a Claimant),
    /// More than one admitted claimant carries the id, in path-sorted
    /// order.
    Ambiguous(Vec<&'a Claimant>),
}

impl Catalog {
    /// Build the catalog for the repository at `root` from one harvest
    /// pass, assessing every claimant against the canonical parse of its
    /// managed position.
    pub(crate) fn build(root: &Path) -> Catalog {
        let mut claimants: Vec<Claimant> = harvest(root)
            .into_iter()
            .map(|claim| {
                let disposition = assess(root, &claim);
                Claimant { claim, disposition }
            })
            .collect();
        claimants.sort_by(|a, b| a.claim.path.cmp(&b.claim.path));
        Catalog { claimants }
    }

    /// Every admitted claimant in path-sorted order.
    pub(crate) fn claimants(&self) -> &[Claimant] {
        &self.claimants
    }

    /// Classify one decoded id against every admitted claimant.
    pub(crate) fn resolve(&self, id: &str) -> Resolution<'_> {
        let matches: Vec<&Claimant> = self
            .claimants
            .iter()
            .filter(|claimant| claimant.claim.id == id)
            .collect();
        match matches.as_slice() {
            [] => Resolution::Missing,
            [only] => Resolution::Unique(only),
            _ => Resolution::Ambiguous(matches),
        }
    }

    /// Every id claimed by more than one admitted claimant, with the
    /// claimants of each in path-sorted order.
    pub(crate) fn ambiguous_ids(&self) -> Vec<(&str, Vec<&Claimant>)> {
        let mut by_id: BTreeMap<&str, Vec<&Claimant>> = BTreeMap::new();
        for claimant in &self.claimants {
            by_id
                .entry(claimant.claim.id.as_str())
                .or_default()
                .push(claimant);
        }
        by_id
            .into_iter()
            .filter(|(_, claimants)| claimants.len() > 1)
            .collect()
    }
}

/// Canonical-parse verdict for one claim, derived by running the exact
/// strict parser the scanners and doctor use on the claimant's managed
/// position. A claimant outside every managed position has no verdict.
fn assess(root: &Path, claim: &Harvested) -> Disposition {
    match canonical_parse(root, &claim.path) {
        None => Disposition::Unassessed,
        Some(Ok(())) => Disposition::Canonical,
        Some(Err(error)) => Disposition::Rejected {
            class: match error {
                crate::error::Error::ArtifactConflict { .. } => "artifact-conflict",
                crate::error::Error::Filesystem { .. } => "unreadable-artifact",
                _ => "malformed-artifact",
            },
        },
    }
}

/// Run the canonical artifact parse for the managed position at
/// repository-relative `path_rel`, or `None` when that path is not a
/// managed artifact position (an unmanaged collection, or a structurally
/// malformed location the strong scanners never assess). Reusing the
/// `read` parsers keeps this verdict in agreement with scanning and
/// diagnosis.
fn canonical_parse(root: &Path, path_rel: &str) -> Option<Result<(), crate::error::Error>> {
    let path = root.join(path_rel);
    for collection in [&crate::read::DRAGON, &crate::read::IDEA] {
        let Some(name) = path_rel
            .strip_prefix(collection.dir)
            .and_then(|rest| rest.strip_prefix('/'))
        else {
            continue;
        };
        if !name.contains('/') {
            return Some(
                crate::read::parse_artifact(&path, collection.dir, name, collection).map(drop),
            );
        }
    }
    let rest = path_rel
        .strip_prefix(crate::repo::SPRINTS_DIR)?
        .strip_prefix('/')?;
    let (dir_name, file_name) = rest.split_once('/')?;
    if file_name.contains('/') {
        return None;
    }
    let sequence = crate::artifact::parse_dir_sequence(dir_name)?;
    if file_name == crate::repo::SPRINT_FILE {
        Some(
            crate::read::parse_artifact_at(
                &path,
                path_rel,
                sequence,
                "the containment directory name",
                &crate::read::SPRINT,
            )
            .map(drop),
        )
    } else {
        Some(
            crate::read::parse_artifact(
                &path,
                &format!("{}/{dir_name}", crate::repo::SPRINTS_DIR),
                file_name,
                &crate::read::TASK,
            )
            .map(drop),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_marker_accepts_bound_and_sugar_forms() {
        assert_eq!(
            parse_marker("[[dec-reference-syntax|reference syntax]]"),
            Some(Marker::Bound {
                id: "dec-reference-syntax",
                label: "reference syntax",
            })
        );
        assert_eq!(
            parse_marker("[[dragon:3]]"),
            Some(Marker::Sugar {
                reference: "dragon:3",
                label: None,
            })
        );
        assert_eq!(
            parse_marker("[[idea:12|relevance surfacing]]"),
            Some(Marker::Sugar {
                reference: "idea:12",
                label: Some("relevance surfacing"),
            })
        );
        // Labels may contain anything but `]]` and newlines.
        assert_eq!(
            parse_marker("[[drg_01KY|open <-> closed, and: more]]"),
            Some(Marker::Bound {
                id: "drg_01KY",
                label: "open <-> closed, and: more",
            })
        );
    }

    #[test]
    fn parse_marker_rejects_non_markers() {
        for bad in [
            "",
            "plain prose",
            "[[]]",
            "[[|label]]",
            "[[id|]]",
            "[[dec-x]]",             // bound requires a label
            "[[two words|label]]",   // whitespace in target
            "[[dec-x#fragment|l]]",  // `#` reserved for fragments
            "[[dec-x|a]]b]]",        // stray closing delimiter
            "[dec-x|label]",         // single brackets
            "[[dec-x|line\nbreak]]", // markers are one line
        ] {
            assert_eq!(parse_marker(bad), None, "must reject {bad:?}");
        }
    }

    fn seed_md(root: &Path, rel_dir: &str, name: &str, id: &str, kind: &str) {
        let dir = root.join(rel_dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(name),
            format!(
                "---\nid: {id}\nsequence: 1\nkind: {kind}\nstatus: accepted\ncreated: 2026-07-20\n---\n\n# {id}\n"
            ),
        )
        .unwrap();
    }

    fn harvested_ids(root: &Path) -> Vec<String> {
        harvest(root).into_iter().map(|h| h.id).collect()
    }

    #[test]
    fn harvest_retains_every_claimant_and_the_catalog_classifies_ambiguity() {
        // Task 23: the harvest keeps every claimant, and the catalog
        // classifies a multiply claimed id as ambiguous rather than
        // collapsing it first-seen-wins.
        let tmp = tempfile::tempdir().unwrap();
        seed_md(
            tmp.path(),
            "archaeology/decisions",
            "0001-a.md",
            "dec-x",
            "decision",
        );
        seed_md(
            tmp.path(),
            "archaeology/notes",
            "0001-b.md",
            "dec-x",
            "decision",
        );

        assert_eq!(harvested_ids(tmp.path()), vec!["dec-x", "dec-x"]);
        let catalog = Catalog::build(tmp.path());
        let Resolution::Ambiguous(claimants) = catalog.resolve("dec-x") else {
            panic!("two admitted claimants must resolve as ambiguous");
        };
        assert_eq!(
            claimant_paths(&claimants),
            vec![
                "archaeology/decisions/0001-a.md",
                "archaeology/notes/0001-b.md",
            ]
        );
    }

    #[test]
    fn harvest_skips_oversized_files_through_the_bounded_seam() {
        let tmp = tempfile::tempdir().unwrap();
        seed_md(
            tmp.path(),
            "archaeology/decisions",
            "0001-a.md",
            "dec-fine",
            "decision",
        );
        let mut oversized = "---\nid: dec-huge\nkind: decision\n---\n\n# Huge\n".to_string();
        oversized.push_str(&"x".repeat(crate::read::MAX_ARTIFACT_BYTES as usize));
        fs::write(
            tmp.path().join("archaeology/decisions/0002-huge.md"),
            &oversized,
        )
        .unwrap();

        assert_eq!(harvested_ids(tmp.path()), vec!["dec-fine"]);
    }

    #[cfg(unix)]
    mod symlink_boundary {
        use super::*;
        use std::os::unix::fs::symlink;

        #[test]
        fn harvest_never_descends_a_symlinked_directory() {
            // Thread 4 claim 4: an external archaeology tree reachable
            // only through a directory symlink must not enter the
            // verification universe.
            let tmp = tempfile::tempdir().unwrap();
            seed_md(
                tmp.path(),
                "archaeology/decisions",
                "0001-a.md",
                "dec-local",
                "decision",
            );
            let outside = tempfile::tempdir().unwrap();
            seed_md(
                outside.path(),
                "external-arch",
                "0001-b.md",
                "dec-external-authority",
                "decision",
            );
            symlink(
                outside.path().join("external-arch"),
                tmp.path().join("archaeology/imported"),
            )
            .unwrap();

            assert_eq!(harvested_ids(tmp.path()), vec!["dec-local"]);
        }

        #[test]
        fn harvest_never_reads_a_symlinked_file() {
            let tmp = tempfile::tempdir().unwrap();
            seed_md(
                tmp.path(),
                "archaeology/decisions",
                "0001-a.md",
                "dec-local",
                "decision",
            );
            let outside = tempfile::tempdir().unwrap();
            seed_md(
                outside.path(),
                "elsewhere",
                "0001-b.md",
                "dec-outside",
                "decision",
            );
            symlink(
                outside.path().join("elsewhere/0001-b.md"),
                tmp.path().join("archaeology/decisions/0002-b.md"),
            )
            .unwrap();

            assert_eq!(harvested_ids(tmp.path()), vec!["dec-local"]);
        }

        #[test]
        fn harvest_ignores_ancestor_loops_without_a_visited_set() {
            // A link back to an ancestor is simply never followed, so the
            // walk cannot cycle regardless of canonicalization.
            let tmp = tempfile::tempdir().unwrap();
            seed_md(
                tmp.path(),
                "archaeology/decisions",
                "0001-a.md",
                "dec-local",
                "decision",
            );
            symlink(
                tmp.path().join("archaeology"),
                tmp.path().join("archaeology/loop"),
            )
            .unwrap();

            assert_eq!(harvested_ids(tmp.path()), vec!["dec-local"]);
        }

        #[test]
        fn harvest_ignores_a_symlinked_archaeology_root() {
            let tmp = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            seed_md(
                outside.path(),
                "tree",
                "0001-a.md",
                "dec-outside",
                "decision",
            );
            symlink(outside.path().join("tree"), tmp.path().join("archaeology")).unwrap();

            assert!(harvested_ids(tmp.path()).is_empty());
        }
    }

    #[test]
    fn claim_admission_threshold_is_exact() {
        // Decision 12: a claim requires bounded UTF-8, valid framing,
        // parseable YAML, and string `id` and `kind`. Nothing below the
        // threshold fabricates a claim; nothing above it is filtered.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("archaeology/notes");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("0001-claim.md"),
            "---\nid: note-claim\nkind: note\n---\n\n# A claim\n",
        )
        .unwrap();
        fs::write(
            dir.join("0002-kindless.md"),
            "---\nid: note-kindless\n---\n",
        )
        .unwrap();
        fs::write(
            dir.join("0003-numeric-kind.md"),
            "---\nid: note-numeric\nkind: 3\n---\n",
        )
        .unwrap();
        fs::write(
            dir.join("0004-bad-yaml.md"),
            "---\nid: [unclosed\nkind: note\n---\n",
        )
        .unwrap();
        fs::write(
            dir.join("0005-unframed.md"),
            "id: note-unframed\nkind: note\n",
        )
        .unwrap();
        fs::write(
            dir.join("0006-not-utf8.md"),
            b"---\nid: note-bytes\nkind: note\n---\n\xff\xfe\n",
        )
        .unwrap();

        assert_eq!(harvested_ids(tmp.path()), vec!["note-claim"]);
    }

    #[test]
    fn quoted_and_unquoted_id_spellings_claim_the_same_decoded_identity() {
        // Decision 12: collision semantics operate on the decoded value;
        // YAML quoting is not a distinct identity.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("archaeology/notes");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("0001-plain.md"),
            "---\nid: dec-twin\nkind: decision\n---\n\n# Plain\n",
        )
        .unwrap();
        fs::write(
            dir.join("0002-quoted.md"),
            "---\nid: \"dec-twin\"\nkind: decision\n---\n\n# Quoted\n",
        )
        .unwrap();

        let catalog = Catalog::build(tmp.path());
        let Resolution::Ambiguous(claimants) = catalog.resolve("dec-twin") else {
            panic!("the two spellings must collide on the decoded id");
        };
        assert_eq!(
            claimant_paths(&claimants),
            vec![
                "archaeology/notes/0001-plain.md",
                "archaeology/notes/0002-quoted.md",
            ]
        );
    }

    #[test]
    fn dispositions_are_explicit_and_not_inferred_from_optional_fields() {
        let tmp = tempfile::tempdir().unwrap();
        // Canonical: a valid dragon at its managed position.
        let dragons = tmp.path().join("archaeology/dragons");
        fs::create_dir_all(&dragons).unwrap();
        fs::write(
            dragons.join("0001-fine.md"),
            "---\nid: drg-fine\nsequence: 1\nkind: dragon\nstatus: open\ncreated: 2026-07-20\n---\n\n# Fine\n",
        )
        .unwrap();
        // Rejected: a dragon whose canonical parse refuses the status,
        // even though `sequence` and `title` are fully recoverable —
        // the disposition is the parser's verdict, not a guess from
        // optional fields.
        fs::write(
            dragons.join("0002-broken.md"),
            "---\nid: drg-broken\nsequence: 2\nkind: dragon\nstatus: done\ncreated: 2026-07-20\n---\n\n# Broken\n",
        )
        .unwrap();
        // Unassessed: an unmanaged decision carrying every optional
        // field; no canonical parse exists for its position.
        seed_md(
            tmp.path(),
            "archaeology/decisions",
            "0001-a.md",
            "dec-a",
            "decision",
        );

        let catalog = Catalog::build(tmp.path());
        let by_id = |id: &str| match catalog.resolve(id) {
            Resolution::Unique(claimant) => claimant.clone(),
            _ => panic!("`{id}` must be unique"),
        };
        assert_eq!(by_id("drg-fine").disposition, Disposition::Canonical);
        let broken = by_id("drg-broken");
        assert_eq!(
            broken.disposition,
            Disposition::Rejected {
                class: "malformed-artifact"
            }
        );
        assert_eq!(broken.claim.sequence, Some(2), "fields stay recoverable");
        assert_eq!(broken.claim.title.as_deref(), Some("Broken"));
        assert_eq!(by_id("dec-a").disposition, Disposition::Unassessed);
        assert!(matches!(catalog.resolve("absent"), Resolution::Missing));
    }

    #[test]
    fn catalog_order_is_path_sorted_under_opposite_creation_orders() {
        // Decision 12's determinism pin: the same relative path set,
        // created in opposite orders in two repositories, yields the same
        // path-sorted claimant and candidate lists.
        let specs = [
            ("archaeology/a-notes", "0001-x.md"),
            ("archaeology/m-notes", "0001-x.md"),
            ("archaeology/z-notes", "0001-x.md"),
        ];
        let build = |order: &[usize]| {
            let tmp = tempfile::tempdir().unwrap();
            for &index in order {
                let (dir, name) = specs[index];
                seed_md(tmp.path(), dir, name, "shared-id", "decision");
            }
            let catalog = Catalog::build(tmp.path());
            let all: Vec<String> = catalog
                .claimants()
                .iter()
                .map(|claimant| claimant.claim.path.clone())
                .collect();
            let Resolution::Ambiguous(claimants) = catalog.resolve("shared-id") else {
                panic!("three claimants must be ambiguous");
            };
            (all, claimant_paths(&claimants))
        };

        let forward = build(&[0, 1, 2]);
        let reverse = build(&[2, 1, 0]);
        let sorted = vec![
            "archaeology/a-notes/0001-x.md".to_string(),
            "archaeology/m-notes/0001-x.md".to_string(),
            "archaeology/z-notes/0001-x.md".to_string(),
        ];
        assert_eq!(forward.0, sorted);
        assert_eq!(forward, reverse);
    }

    #[test]
    fn vocabulary_never_admits_idea_targets() {
        for kind in EDGE_KINDS {
            assert!(
                !kind.target_kinds.contains(&"idea"),
                "`{}` must not target ideas",
                kind.key
            );
        }
    }
}
