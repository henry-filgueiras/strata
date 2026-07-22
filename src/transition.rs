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
    let artifacts = read::scan_collection(root, collection)?;
    let artifact = read::resolve(&artifacts, target, display)?;
    perform(root, collection, artifact, to)
}

/// Close one sprint, refusing while it still has pending tasks.
///
/// The refusal names each pending task, because the caller's next move is
/// to close or reassign them; an empty sprint closes like any other
/// artifact, stamping its `closed:` date.
pub fn close_sprint(root: &Path, target: Selector<'_>, display: &str) -> Result<Transition, Error> {
    let sprints = read::scan_sprints(root)?;
    let sprint = read::resolve(&sprints, target, display)?;
    let pending: Vec<String> = read::scan_tasks(root)?
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

/// Rewrite exactly the front-matter `status` value from `from` to `to`,
/// preserving every other byte.
///
/// Only the front-matter block is touched; `status:` lines in the body are
/// content. The value must be written in the plain unquoted form the
/// templates produce — exactly one top-level `status:` line whose trimmed
/// value equals `from`. Anything else (quoted values, trailing comments,
/// duplicated keys) is refused rather than guessed at, since a wrong splice
/// would corrupt canonical bytes.
fn rewrite_status(content: &str, from: Status, to: Status) -> Result<String, String> {
    let (fm_start, fm_end) = front_matter_region(content)
        .ok_or_else(|| "missing front matter: cannot locate the `status` value".to_string())?;
    let front_matter = &content[fm_start..fm_end];

    let mut value_range = None;
    let mut offset = 0;
    for line in front_matter.split_inclusive('\n') {
        let line_text = line.strip_suffix('\n').unwrap_or(line);
        if let Some(value) = line_text.strip_prefix("status:") {
            let trimmed = value.trim();
            if trimmed != from.name() {
                return Err(format!(
                    "the front-matter `status` value is not written as plain \
                     `{}`, so Strata cannot rewrite it precisely; edit the \
                     file by hand",
                    from.name()
                ));
            }
            if value_range.is_some() {
                return Err(
                    "multiple front-matter `status` lines; repair the file by hand".to_string(),
                );
            }
            let leading = value.len() - value.trim_start().len();
            let start = fm_start + offset + "status:".len() + leading;
            value_range = Some(start..start + trimmed.len());
        }
        offset += line.len();
    }

    let range = value_range.ok_or_else(|| {
        format!(
            "no plain front-matter `status: {}` line found to rewrite; edit \
             the file by hand",
            from.name()
        )
    })?;
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
    fn rewrite_status_refuses_quoted_or_commented_values() {
        for status_line in ["status: \"open\"", "status: 'open'", "status: open # note"] {
            let content = format!("---\nid: x\n{status_line}\n---\n\n# T\n");
            let err = rewrite_status(&content, Status::Open, Status::Closed).unwrap_err();
            assert!(err.contains("by hand"), "for {status_line:?}: {err}");
        }
    }

    #[test]
    fn rewrite_status_refuses_when_no_status_line_exists() {
        let err =
            rewrite_status("---\nid: x\n---\n\n# T\n", Status::Open, Status::Closed).unwrap_err();
        assert!(err.contains("no plain front-matter"), "{err}");
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
