//! Lifecycle transitions for managed collections.
//!
//! A transition rewrites exactly the artifact's front-matter `status`
//! value; every other byte is preserved and the file never moves —
//! placement is flat per decision 11, so front matter is the sole
//! lifecycle authority. The mutation is one safe write under the
//! decision 8 failure classes: the full payload is staged in a temporary
//! beside the artifact and atomically renamed over it, so a returned error
//! leaves the original bytes untouched and an abrupt termination leaves
//! exactly one valid artifact at its one path. The former two-step
//! contract (rewrite, then file into a lifecycle directory) is retired
//! with the lifecycle directories themselves.
//!
//! # Concurrency boundary
//!
//! Bootstrap does not linearize concurrent Strata processes: two
//! processes rewriting the same artifact race last-write-wins, matching
//! creation's scan-then-write posture.

use std::path::Path;

use crate::error::Error;
use crate::read::{self, Artifact, Collection, Selector, Status};

/// A successfully performed lifecycle transition.
#[derive(Debug)]
pub struct Transition {
    /// Human reference of the artifact, e.g. `dragon:2`.
    pub reference: String,
    /// Lifecycle state before the transition.
    pub from: Status,
    /// Lifecycle state after the transition.
    pub to: Status,
    /// Repository-relative path of the artifact, unchanged by the
    /// transition, with `/` separators.
    pub path: String,
}

/// Transition one artifact of `collection` to `to`, resolving `target` like
/// `strata show`.
///
/// Resolution reuses the strict read pipeline, so a malformed artifact
/// refuses with a typed error naming the file. A transition the
/// collection's lifecycle does not define (such as un-adopting an idea) is
/// an invalid invocation.
pub fn transition(
    root: &Path,
    collection: &Collection,
    target: Selector<'_>,
    display: &str,
    to: Status,
) -> Result<Transition, Error> {
    transition_with_provenance(root, collection, target, display, to, None)
}

/// Transition one artifact and, when `edge` is given, write the named
/// provenance edge in the same atomic write: no transition without its
/// edge, no edge without its transition.
///
/// `edge` is `(front-matter key, raw target)`; the raw target may be a
/// stable id or a `kind:N` sequence reference, and is resolved to a bound
/// marker — stable id plus the target's title as the frozen label — at
/// write time, per decision 10.
pub fn transition_with_provenance(
    root: &Path,
    collection: &Collection,
    target: Selector<'_>,
    display: &str,
    to: Status,
    edge: Option<(&'static str, &str)>,
) -> Result<Transition, Error> {
    let artifacts = read::scan_collection(root, collection).map_err(|err| err.blocking(display))?;
    let artifact = read::resolve(&artifacts, target, display)?;
    let edge_line = edge
        .map(|(key, raw)| resolve_edge(root, key, raw))
        .transpose()?;
    perform_with_edge(root, collection, artifact, to, edge_line)
}

/// Resolve one provenance-flag target to its front-matter line.
///
/// Both target forms resolve through the identity claimant catalog
/// (task 23): a stable id claimed by more than one artifact is refused as
/// `ambiguous-reference` naming every claimant path, before any mutation,
/// exactly as the `kind:N` form already refuses duplicated sequences.
/// A unique target must also yield a parseable bound marker (decision 12):
/// an unaddressable target id or an unrepresentable frozen title is
/// refused, naming the offending character class, before any mutation.
fn resolve_edge(root: &Path, key: &'static str, raw: &str) -> Result<(String, String), Error> {
    let kind = crate::edges::EDGE_KINDS
        .iter()
        .find(|kind| kind.key == key)
        .expect("provenance flags only carry decided edge keys");
    let catalog = crate::edges::Catalog::build(root);
    let target = if let Some((target_kind, sequence)) = raw.split_once(':') {
        let sequence: u32 = sequence.parse().map_err(|_| Error::InvalidInvocation {
            message: format!(
                "invalid sequence in `--{key} {raw}`; expected `kind:N` or a \
                 stable artifact id"
            ),
        })?;
        let matches: Vec<&crate::edges::Claimant> = catalog
            .claimants()
            .iter()
            .filter(|claimant| {
                claimant.claim.kind == target_kind && claimant.claim.sequence == Some(sequence)
            })
            .collect();
        match matches.as_slice() {
            [] => {
                return Err(Error::ArtifactNotFound {
                    reference: raw.to_string(),
                });
            }
            [only] => only.claim.clone(),
            several => {
                return Err(Error::AmbiguousReference {
                    reference: raw.to_string(),
                    candidates: several
                        .iter()
                        .map(|claimant| claimant.claim.path.clone())
                        .collect(),
                });
            }
        }
    } else {
        match catalog.resolve(raw) {
            crate::edges::Resolution::Missing => {
                return Err(Error::ArtifactNotFound {
                    reference: raw.to_string(),
                });
            }
            crate::edges::Resolution::Unique(claimant) => claimant.claim.clone(),
            crate::edges::Resolution::Ambiguous(claimants) => {
                return Err(Error::AmbiguousReference {
                    reference: raw.to_string(),
                    candidates: claimants
                        .iter()
                        .map(|claimant| claimant.claim.path.clone())
                        .collect(),
                });
            }
        }
    };
    if !kind.target_kinds.contains(&target.kind.as_str()) {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`--{key}` targets `{}`, a {}; legal targets are: {}",
                target.id,
                target.kind,
                kind.target_kinds.join(", ")
            ),
        });
    }
    let Some(title) = target.title else {
        return Err(Error::MalformedArtifact {
            path: root.join(&target.path),
            reason: format!(
                "cannot freeze a label for `--{key} {raw}`: the target has \
                 no readable title heading"
            ),
        });
    };
    // Decision 12: validate the constructed semantic marker — the decoded
    // id and the frozen title — through the one marker parser before any
    // mutation; YAML carrier escaping happens after and changes neither.
    if let Err(violation) = crate::edges::addressable(&target.id) {
        return Err(Error::MalformedArtifact {
            path: root.join(&target.path),
            reason: format!(
                "cannot bind `--{key} {raw}`: the target's stable id `{}` \
                 {}, so it is not addressable as a bound-marker target",
                target.id,
                violation.describe()
            ),
        });
    }
    if let Err(violation) = crate::edges::label_valid(&title) {
        return Err(Error::MalformedArtifact {
            path: root.join(&target.path),
            reason: format!(
                "cannot freeze a label for `--{key} {raw}`: the target's \
                 title {}",
                violation.describe()
            ),
        });
    }
    let marker = format!("[[{}|{title}]]", target.id);
    match crate::edges::parse_marker(&marker) {
        Some(crate::edges::Marker::Bound { id, label }) if id == target.id && label == title => {}
        _ => {
            return Err(Error::MalformedArtifact {
                path: root.join(&target.path),
                reason: format!(
                    "cannot bind `--{key} {raw}`: the constructed marker \
                     `{marker}` does not round-trip through the reference \
                     grammar"
                ),
            });
        }
    }
    let label = title.replace('\\', "\\\\").replace('"', "\\\"");
    Ok((key.to_string(), format!("\"[[{}|{label}]]\"", target.id)))
}

/// Close one sprint, refusing while it still has pending tasks.
///
/// The refusal names each pending task, because the caller's next move is
/// to close or reassign them; an empty sprint closes like any other
/// artifact, stamping its `closed:` date.
pub fn close_sprint(root: &Path, target: Selector<'_>, display: &str) -> Result<Transition, Error> {
    let sprints = read::scan_sprints(root).map_err(|err| err.blocking(display))?;
    let sprint = read::resolve(&sprints, target, display)?;
    let pending: Vec<String> = read::scan_tasks(root)
        .map_err(|err| err.blocking(display))?
        .iter()
        .filter(|task| {
            task.summary.status == Status::Pending
                && task.summary.sprint.as_deref() == Some(sprint.summary.id.as_str())
        })
        .map(|task| format!("{} ({})", task.summary.reference(), task.summary.title))
        .collect();
    if !pending.is_empty() {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`{}` still has {} pending task(s): {}; close them first",
                sprint.summary.reference(),
                pending.len(),
                pending.join(", ")
            ),
        });
    }
    perform(root, &read::SPRINT, sprint, Status::Closed)
}

/// Perform the transition of one resolved artifact.
pub(crate) fn perform(
    root: &Path,
    collection: &Collection,
    artifact: &Artifact,
    to: Status,
) -> Result<Transition, Error> {
    perform_with_edge(root, collection, artifact, to, None)
}

/// Perform the transition of one resolved artifact, optionally writing a
/// resolved provenance edge line in the same write.
pub(crate) fn perform_with_edge(
    root: &Path,
    collection: &Collection,
    artifact: &Artifact,
    to: Status,
    edge_line: Option<(String, String)>,
) -> Result<Transition, Error> {
    let from = artifact.summary.status;
    let reference = artifact.summary.reference();
    let path_rel = &artifact.summary.path;
    if from == to {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`{reference}` is already {from} (at `{path_rel}`); no transition needed"
            ),
        });
    }
    if !collection.allows(from, to) {
        return Err(Error::InvalidInvocation {
            message: format!(
                "`{reference}` is {from}, and `{from} -> {to}` is not a {} \
                 transition; the {} lifecycle is: {}",
                collection.kind,
                collection.kind,
                collection.transition_names()
            ),
        });
    }

    let path = root.join(path_rel);
    let mut rewritten =
        rewrite_status(&artifact.content, from, to).map_err(|reason| Error::MalformedArtifact {
            path: path.clone(),
            reason,
        })?;
    if to == Status::Closed && collection.stamp_closed {
        let today = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();
        rewritten =
            stamp_closed(&rewritten, &today).map_err(|reason| Error::MalformedArtifact {
                path: path.clone(),
                reason,
            })?;
    }
    if let Some((key, value)) = edge_line {
        rewritten = insert_after_created(&rewritten, &key, &value).map_err(|reason| {
            Error::MalformedArtifact {
                path: path.clone(),
                reason,
            }
        })?;
    }

    replace(&path, &rewritten).map_err(|source| Error::Filesystem {
        operation: "rewrite front-matter status".into(),
        path,
        source,
    })?;

    Ok(Transition {
        reference,
        from,
        to,
        path: path_rel.clone(),
    })
}

/// Atomically replace `dest` with `content` via an exclusive temporary
/// file beside it. Clobbering is intended: `dest` is the artifact being
/// rewritten in place. A failure at any point leaves `dest` untouched.
fn replace(dest: &Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    let dir = dest.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "artifact path has no parent",
        )
    })?;
    let mut tmp = tempfile::Builder::new()
        .prefix(".strata.artifact.tmp")
        .tempfile_in(dir)?;
    tmp.write_all(content.as_bytes())?;
    tmp.persist(dest).map_err(|err| err.error)?;
    Ok(())
}

/// Locate the canonical mutable `status` carrier, per decision 12
/// (`dec-canonical-representation`): within the front-matter block,
/// exactly one line beginning `status:` at column zero whose trimmed
/// remainder is exactly the artifact's admitted lowercase status
/// (`expected`). Whitespace around the value is accepted; quoting, inline
/// comments, duplicate top-level carriers, and any other spelling are not
/// canonical carriers. Indented spellings never match.
///
/// Returns the byte range of the semantic value inside `content`, so a
/// splice replaces exactly the value and preserves every surrounding
/// byte. This is the one lexical recognizer shared by the transition
/// splicer and doctor, so the two surfaces cannot drift; each refusal
/// names the canonical spelling.
///
/// This judges representation conformance only. The canonical parse may
/// still deserialize a quoted or comment-bearing status semantically;
/// doctor reports such carriers as noncanonical and transitions refuse
/// them before writing.
pub(crate) fn canonical_status_carrier(
    content: &str,
    expected: &str,
) -> Result<std::ops::Range<usize>, String> {
    let (fm_start, fm_end) = front_matter_region(content)
        .ok_or_else(|| "missing front matter: cannot locate the `status` value".to_string())?;
    let front_matter = &content[fm_start..fm_end];

    let mut carriers = Vec::new();
    let mut offset = 0;
    for line in front_matter.split_inclusive('\n') {
        let line_text = line.strip_suffix('\n').unwrap_or(line);
        if let Some(value) = line_text.strip_prefix("status:") {
            carriers.push((offset, value));
        }
        offset += line.len();
    }

    let (line_offset, value) = match carriers.as_slice() {
        [] => {
            return Err(format!(
                "no top-level front-matter `status:` line found; the \
                 canonical mutable carrier is exactly `status: {expected}`"
            ));
        }
        [only] => *only,
        _ => {
            return Err(format!(
                "multiple top-level front-matter `status:` lines; exactly \
                 one canonical `status: {expected}` carrier is allowed"
            ));
        }
    };

    let trimmed = value.trim();
    if trimmed != expected {
        let class = if trimmed.starts_with('"') || trimmed.starts_with('\'') {
            "is quoted"
        } else if trimmed
            .strip_prefix(expected)
            .is_some_and(|rest| rest.trim_start().starts_with('#'))
        {
            "carries an inline comment"
        } else {
            "is not the plain admitted status word"
        };
        return Err(format!(
            "the front-matter `status` value {class}; the canonical mutable \
             carrier is exactly `status: {expected}`"
        ));
    }
    let leading = value.len() - value.trim_start().len();
    let start = fm_start + line_offset + "status:".len() + leading;
    Ok(start..start + trimmed.len())
}

/// Rewrite exactly the front-matter `status` value from `from` to `to`,
/// preserving every other byte.
///
/// Only the front-matter block is touched; `status:` lines in the body are
/// content. The carrier must be the canonical form the templates produce,
/// judged by [`canonical_status_carrier`]; anything else is refused rather
/// than guessed at, since a wrong splice would corrupt canonical bytes.
fn rewrite_status(content: &str, from: Status, to: Status) -> Result<String, String> {
    let range = canonical_status_carrier(content, from.name())?;
    let mut rewritten = String::with_capacity(content.len() + to.name().len());
    rewritten.push_str(&content[..range.start]);
    rewritten.push_str(to.name());
    rewritten.push_str(&content[range.end..]);
    Ok(rewritten)
}

/// Insert a `closed: <date>` line after the front-matter `created:` line,
/// as part of the same transition write. A `closed:` line already present
/// (a hand-authored record) is left untouched.
fn stamp_closed(content: &str, date: &str) -> Result<String, String> {
    let (fm_start, fm_end) =
        front_matter_region(content).ok_or_else(|| "missing front matter".to_string())?;
    let front_matter = &content[fm_start..fm_end];
    if front_matter.lines().any(|line| line.starts_with("closed:")) {
        return Ok(content.to_string());
    }
    let mut offset = 0;
    for line in front_matter.split_inclusive('\n') {
        offset += line.len();
        if line.starts_with("created:") {
            let insert_at = fm_start + offset;
            let mut stamped = String::with_capacity(content.len() + date.len() + 9);
            stamped.push_str(&content[..insert_at]);
            stamped.push_str("closed: ");
            stamped.push_str(date);
            stamped.push('\n');
            stamped.push_str(&content[insert_at..]);
            return Ok(stamped);
        }
    }
    Err(
        "no front-matter `created:` line to stamp `closed:` after; edit the \
         file by hand"
            .to_string(),
    )
}

/// Insert `key: value` after the front-matter `created:` line, refusing
/// when the key is already present: an existing edge is authored history
/// the command must not silently rewrite.
fn insert_after_created(content: &str, key: &str, value: &str) -> Result<String, String> {
    let (fm_start, fm_end) =
        front_matter_region(content).ok_or_else(|| "missing front matter".to_string())?;
    let front_matter = &content[fm_start..fm_end];
    if front_matter.lines().any(|line| {
        line.strip_prefix(key)
            .is_some_and(|rest| rest.starts_with(':'))
    }) {
        return Err(format!(
            "the front matter already carries a `{key}` edge; edit the file \
             by hand to change recorded provenance"
        ));
    }
    let mut offset = 0;
    for line in front_matter.split_inclusive('\n') {
        offset += line.len();
        if line.starts_with("created:") {
            let insert_at = fm_start + offset;
            let mut inserted = String::with_capacity(content.len() + key.len() + value.len() + 3);
            inserted.push_str(&content[..insert_at]);
            inserted.push_str(key);
            inserted.push_str(": ");
            inserted.push_str(value);
            inserted.push('\n');
            inserted.push_str(&content[insert_at..]);
            return Ok(inserted);
        }
    }
    Err(format!(
        "no front-matter `created:` line to place the `{key}` edge after; \
         edit the file by hand"
    ))
}

/// Byte range of the front-matter block body, mirroring the read pipeline's
/// delimiter rules: the file opens with `---\n` and the block ends at the
/// next `---` line.
fn front_matter_region(content: &str) -> Option<(usize, usize)> {
    let rest = content.strip_prefix("---\n")?;
    if let Some(end) = rest.find("\n---\n") {
        Some((4, 4 + end + 1))
    } else if rest.ends_with("\n---") {
        Some((4, content.len() - 3))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo;
    use crate::repo::{DRAGONS_DIR, IDEAS_DIR};
    use std::fs;

    fn temp_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create temporary directory");
        repo::init(tmp.path()).expect("initialize repository");
        tmp
    }

    fn rich_dragon(status: &str) -> String {
        format!(
            "---\nid: drg-rich\nsequence: 1\nkind: dragon\nstatus: {status}\ncreated: 2026-07-20\nseverity: high\n---\n\n# Rich dragon\n\nBody mentions status: open in prose.  \n\n```yaml\nstatus: open\n```\n"
        )
    }

    fn seed(root: &Path, dir: &str, name: &str, content: &str) {
        fs::create_dir_all(root.join(dir)).unwrap();
        fs::write(root.join(dir).join(name), content).unwrap();
    }

    #[test]
    fn rewrite_status_changes_exactly_the_status_value() {
        let original = rich_dragon("open");
        let rewritten = rewrite_status(&original, Status::Open, Status::Closed).unwrap();

        assert_eq!(rewritten, rich_dragon("closed"));
        assert!(
            rewritten.contains("Body mentions status: open in prose."),
            "body bytes must be untouched"
        );
        assert!(
            rewritten.contains("```yaml\nstatus: open\n```"),
            "fenced content must be untouched"
        );
    }

    #[test]
    fn rewrite_status_preserves_unusual_spacing_around_the_value() {
        let original = "---\nid: x\nsequence: 1\nkind: dragon\nstatus:   open  \ncreated: 2026-07-20\n---\n\n# T\n";
        let rewritten = rewrite_status(original, Status::Open, Status::Closed).unwrap();
        assert_eq!(
            rewritten,
            "---\nid: x\nsequence: 1\nkind: dragon\nstatus:   closed  \ncreated: 2026-07-20\n---\n\n# T\n"
        );
    }

    #[test]
    fn rewrite_status_refuses_quoted_or_commented_values_naming_the_canonical_spelling() {
        // Task 25: the refusal names the canonical spelling instead of
        // deferring to a doctor that (formerly) reported nothing.
        for (status_line, class) in [
            ("status: \"open\"", "quoted"),
            ("status: 'open'", "quoted"),
            ("status: open # note", "inline comment"),
        ] {
            let content = format!("---\nid: x\n{status_line}\n---\n\n# T\n");
            let err = rewrite_status(&content, Status::Open, Status::Closed).unwrap_err();
            assert!(err.contains(class), "for {status_line:?}: {err}");
            assert!(
                err.contains("`status: open`"),
                "the refusal must name the canonical spelling for {status_line:?}: {err}"
            );
        }
    }

    #[test]
    fn rewrite_status_refuses_when_no_status_line_exists() {
        let err =
            rewrite_status("---\nid: x\n---\n\n# T\n", Status::Open, Status::Closed).unwrap_err();
        assert!(err.contains("no top-level front-matter"), "{err}");
        assert!(err.contains("`status: open`"), "{err}");
    }

    #[test]
    fn duplicate_and_indented_status_carriers_are_refused_or_ignored() {
        // Decision 12: duplicate top-level carriers are refused; an
        // indented spelling is not a carrier at all, so only the
        // column-zero line is spliced.
        let duplicated = "---\nid: x\nstatus: open\nstatus: open\n---\n\n# T\n";
        let err = rewrite_status(duplicated, Status::Open, Status::Closed).unwrap_err();
        assert!(err.contains("multiple top-level"), "{err}");

        let indented = "---\nid: x\nstatus: open\nnested:\n  status: parked\n---\n\n# T\n";
        let rewritten = rewrite_status(indented, Status::Open, Status::Closed).unwrap();
        assert_eq!(
            rewritten, "---\nid: x\nstatus: closed\nnested:\n  status: parked\n---\n\n# T\n",
            "the indented spelling is content, not a carrier"
        );
    }

    #[test]
    fn transition_rewrites_in_place_without_moving_the_file() {
        let tmp = temp_repo();
        seed(
            tmp.path(),
            DRAGONS_DIR,
            "0001-rich-dragon.md",
            &rich_dragon("open"),
        );

        let done = transition(
            tmp.path(),
            &read::DRAGON,
            Selector::Sequence(1),
            "dragon:1",
            Status::Closed,
        )
        .unwrap();

        assert_eq!(done.path, format!("{DRAGONS_DIR}/0001-rich-dragon.md"));
        let path = tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md");
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            rich_dragon("closed"),
            "only the status value may change, and the file must not move"
        );
        assert!(crate::doctor::check(tmp.path()).unwrap().healthy());
    }

    #[test]
    fn undefined_transitions_are_invalid_invocations_naming_the_lifecycle() {
        let tmp = temp_repo();
        seed(
            tmp.path(),
            IDEAS_DIR,
            "0001-settled.md",
            "---\nid: idea-settled\nsequence: 1\nkind: idea\nstatus: adopted\ncreated: 2026-07-20\n---\n\n# Settled\n",
        );

        let err = transition(
            tmp.path(),
            &read::IDEA,
            Selector::Sequence(1),
            "idea:1",
            Status::Rejected,
        )
        .unwrap_err();

        assert!(matches!(err, Error::InvalidInvocation { .. }), "{err:?}");
        let message = err.to_string();
        assert!(
            message.contains("parked -> adopted, parked -> rejected"),
            "the refusal must name the legal lifecycle: {message}"
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join(IDEAS_DIR).join("0001-settled.md")).unwrap(),
            "---\nid: idea-settled\nsequence: 1\nkind: idea\nstatus: adopted\ncreated: 2026-07-20\n---\n\n# Settled\n",
            "nothing may change"
        );
    }

    #[test]
    fn stable_id_binding_refuses_an_ambiguous_target_before_mutation() {
        // Task 23: one canonical claimant plus one canonically rejected
        // claimant of the same id is ambiguity, refused with the same
        // `ambiguous-reference` contract as the `kind:N` arm — never
        // resolved to the traversal's first claimant, and never after
        // touching the source artifact.
        let tmp = temp_repo();
        let original = rich_dragon("open");
        seed(tmp.path(), DRAGONS_DIR, "0001-rich-dragon.md", &original);
        // The canonical claimant: a healthy closed task.
        seed(
            tmp.path(),
            "archaeology/sprints/0001-work",
            "0001-target.md",
            "---\nid: dup-target\nsequence: 1\nkind: task\nstatus: closed\nsprint: spr-1\ncreated: 2026-07-20\n---\n\n# Canonical claimant\n",
        );
        // The rejected claimant: an idea with an inadmissible status,
        // refused by canonical parsing yet retained as an admitted
        // claim. It sits outside the scanned dragon collection, so the
        // ambiguity — not the strict sibling scan — is what refuses.
        seed(
            tmp.path(),
            IDEAS_DIR,
            "0001-broken.md",
            "---\nid: dup-target\nsequence: 1\nkind: idea\nstatus: done\ncreated: 2026-07-20\n---\n\n# Rejected claimant\n",
        );

        let err = transition_with_provenance(
            tmp.path(),
            &read::DRAGON,
            Selector::Sequence(1),
            "dragon:1",
            Status::Closed,
            Some(("resolved-by", "dup-target")),
        )
        .unwrap_err();

        let Error::AmbiguousReference { candidates, .. } = err else {
            panic!("expected ambiguous-reference, got {err:?}");
        };
        assert_eq!(
            candidates,
            vec![
                "archaeology/ideas/0001-broken.md".to_string(),
                "archaeology/sprints/0001-work/0001-target.md".to_string(),
            ],
            "candidates must be every claimant in path-sorted order"
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap(),
            original,
            "refusal must precede any mutation"
        );
    }

    fn seed_decision_titled(root: &Path, id: &str, title: &str) {
        seed(
            root,
            "archaeology/decisions",
            "0001-target.md",
            &format!(
                "---\nid: {id}\nsequence: 1\nkind: decision\nstatus: accepted\ncreated: 2026-07-20\n---\n\n# {title}\n"
            ),
        );
    }

    #[test]
    fn binding_refuses_an_unaddressable_unique_target_before_mutation() {
        // Task 25: a unique target whose decoded id fails the decision 12
        // addressability contract is refused, naming the offending class,
        // with the source artifact byte-identical.
        let tmp = temp_repo();
        let original = rich_dragon("open");
        seed(tmp.path(), DRAGONS_DIR, "0001-rich-dragon.md", &original);
        seed_decision_titled(tmp.path(), "dec spacey", "Spacey decision");

        let err = transition_with_provenance(
            tmp.path(),
            &read::DRAGON,
            Selector::Sequence(1),
            "dragon:1",
            Status::Closed,
            Some(("resolved-by", "dec spacey")),
        )
        .unwrap_err();

        let Error::MalformedArtifact { path, reason } = err else {
            panic!("expected malformed-artifact, got {err:?}");
        };
        assert!(path.ends_with("0001-target.md"), "{path:?}");
        assert!(
            reason.contains("whitespace"),
            "must name the class: {reason}"
        );
        assert!(reason.contains("not addressable"), "{reason}");
        assert_eq!(
            fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap(),
            original,
            "refusal must precede any mutation"
        );
    }

    #[test]
    fn binding_refuses_a_double_bracket_title_before_mutation() {
        let tmp = temp_repo();
        let original = rich_dragon("open");
        seed(tmp.path(), DRAGONS_DIR, "0001-rich-dragon.md", &original);
        seed_decision_titled(tmp.path(), "dec-brackets", "The [[worst]] title");

        let err = transition_with_provenance(
            tmp.path(),
            &read::DRAGON,
            Selector::Sequence(1),
            "dragon:1",
            Status::Closed,
            Some(("resolved-by", "dec-brackets")),
        )
        .unwrap_err();

        let Error::MalformedArtifact { reason, .. } = err else {
            panic!("expected malformed-artifact, got {err:?}");
        };
        assert!(reason.contains("`]]`"), "must name the class: {reason}");
        assert_eq!(
            fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap(),
            original,
            "refusal must precede any mutation"
        );
    }

    #[test]
    fn binding_freezes_a_single_bracket_title_that_round_trips() {
        // Decision 12: decision 10 as written — a single `]` in the frozen
        // label is legal, and the constructed marker round-trips through
        // the parser before the write.
        let tmp = temp_repo();
        seed(
            tmp.path(),
            DRAGONS_DIR,
            "0001-rich-dragon.md",
            &rich_dragon("open"),
        );
        seed_decision_titled(tmp.path(), "dec-bracket", "Handle the arr[0] edge case");

        let done = transition_with_provenance(
            tmp.path(),
            &read::DRAGON,
            Selector::Sequence(1),
            "dragon:1",
            Status::Closed,
            Some(("resolved-by", "dec-bracket")),
        )
        .unwrap();

        assert_eq!(done.to, Status::Closed);
        let content =
            fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap();
        assert!(
            content.contains("resolved-by: \"[[dec-bracket|Handle the arr[0] edge case]]\""),
            "{content}"
        );
        // The written marker parses back to exactly the intended id and
        // frozen title, and the repository stays healthy.
        assert_eq!(
            crate::edges::parse_marker("[[dec-bracket|Handle the arr[0] edge case]]"),
            Some(crate::edges::Marker::Bound {
                id: "dec-bracket",
                label: "Handle the arr[0] edge case",
            })
        );
        assert!(crate::doctor::check(tmp.path()).unwrap().healthy());
    }

    #[test]
    fn unique_rejected_claimant_binding_preserves_the_deferred_seam() {
        // Decision 12 records this compatibility behavior as deferred,
        // not desirable: a *unique* claimant whose artifact is rejected
        // by canonical parsing may still serve as a provenance target
        // when a title is extractable. Task 23 preserves it unchanged;
        // its repair belongs to a future decision.
        let tmp = temp_repo();
        seed(
            tmp.path(),
            DRAGONS_DIR,
            "0001-rich-dragon.md",
            &rich_dragon("open"),
        );
        let sprint_dir = tmp.path().join(crate::repo::SPRINTS_DIR).join("0001-work");
        fs::create_dir_all(&sprint_dir).unwrap();
        fs::write(
            sprint_dir.join("0001-broken.md"),
            "---\nid: tsk-broken\nsequence: 1\nkind: task\nstatus: done\nsprint: spr-1\ncreated: 2026-07-20\n---\n\n# Broken task\n",
        )
        .unwrap();

        let done = transition_with_provenance(
            tmp.path(),
            &read::DRAGON,
            Selector::Sequence(1),
            "dragon:1",
            Status::Closed,
            Some(("resolved-by", "tsk-broken")),
        )
        .unwrap();

        assert_eq!(done.to, Status::Closed);
        let content =
            fs::read_to_string(tmp.path().join(DRAGONS_DIR).join("0001-rich-dragon.md")).unwrap();
        assert!(
            content.contains("resolved-by: \"[[tsk-broken|Broken task]]\""),
            "{content}"
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
    fn closing_a_sprint_stamps_the_closed_date() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0001-done", 1, "active");

        let done = close_sprint(tmp.path(), Selector::Sequence(1), "sprint:1").unwrap();

        assert_eq!(done.to, Status::Closed);
        let content = fs::read_to_string(
            tmp.path()
                .join(crate::repo::SPRINTS_DIR)
                .join("0001-done")
                .join(crate::repo::SPRINT_FILE),
        )
        .unwrap();
        assert!(content.contains("status: closed"), "{content}");
        assert!(
            content.contains("\ncreated: 2026-07-20\nclosed: "),
            "the closed stamp must follow the created line: {content}"
        );
    }

    #[test]
    fn closing_a_sprint_with_pending_tasks_is_refused_naming_them() {
        let tmp = temp_repo();
        seed_sprint(tmp.path(), "0001-busy", 1, "active");
        fs::write(
            tmp.path()
                .join(crate::repo::SPRINTS_DIR)
                .join("0001-busy")
                .join("0001-unfinished.md"),
            "---\nid: tsk-unfinished\nsequence: 1\nkind: task\nstatus: pending\nsprint: spr-1\ncreated: 2026-07-20\n---\n\n# Unfinished work\n",
        )
        .unwrap();

        let err = close_sprint(tmp.path(), Selector::Sequence(1), "sprint:1").unwrap_err();

        assert!(matches!(err, Error::InvalidInvocation { .. }), "{err:?}");
        let message = err.to_string();
        assert!(
            message.contains("task:1") && message.contains("Unfinished work"),
            "the refusal must name each pending task: {message}"
        );
    }

    #[test]
    fn stamp_closed_leaves_an_existing_closed_line_untouched() {
        let content = "---\nid: x\nsequence: 1\nkind: sprint\nstatus: closed\ncreated: 2026-07-20\nclosed: 2026-07-21\n---\n\n# T\n";
        assert_eq!(stamp_closed(content, "2026-07-22").unwrap(), content);
    }

    #[cfg(unix)]
    #[test]
    fn failed_write_leaves_the_original_bytes_untouched() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = temp_repo();
        let original = rich_dragon("open");
        seed(tmp.path(), DRAGONS_DIR, "0001-rich-dragon.md", &original);
        let dir = tmp.path().join(DRAGONS_DIR);
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();

        let result = transition(
            tmp.path(),
            &read::DRAGON,
            Selector::Sequence(1),
            "dragon:1",
            Status::Closed,
        );

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            matches!(result, Err(Error::Filesystem { .. })),
            "{result:?}"
        );
        assert_eq!(
            fs::read_to_string(dir.join("0001-rich-dragon.md")).unwrap(),
            original,
            "a failed transition must leave the original bytes"
        );
        assert!(crate::doctor::check(tmp.path()).unwrap().healthy());
    }
}
