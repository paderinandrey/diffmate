use super::caps::{RULE_FILE_CAP, cap_split, reason};
use super::{note_truncation, put};
use crate::error::Error;
use crate::git::{Git, TreeEntry};
use crate::paths::create_private_dir_async;
use std::path::Path;

const FILES: [&str; 4] = ["CLAUDE.md", "AGENTS.md", "CONTRIBUTING.md", "REVIEW.md"];
const LINK_CAP: usize = 4096;

fn plain(target: &str) -> bool {
    let safe = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-');
    target.bytes().all(safe) && !matches!(target, "" | "." | "..")
}

async fn content<G: Git>(
    git: &G,
    repo: &Path,
    (name, entry): (&str, &TreeEntry),
    note: &str,
) -> Result<String, Error> {
    let bytes = git.blob(repo, &entry.oid, RULE_FILE_CAP).await?;
    let text = String::from_utf8_lossy(&bytes);
    let total = usize::try_from(entry.size).unwrap_or(usize::MAX);
    let (marker, kept) = cap_split(&format!("rules/{name}"), &text, total, RULE_FILE_CAP);
    let marker = marker.map(|line| format!("{line}\n")).unwrap_or_default();
    Ok(format!("{marker}{note}{kept}"))
}

async fn symlink<G: Git>(
    git: &G,
    (repo, base): (&Path, &str),
    (name, entry): (&str, &TreeEntry),
) -> Result<String, Error> {
    let bytes = git.blob(repo, &entry.oid, LINK_CAP).await?;
    let target = String::from_utf8_lossy(&bytes).into_owned();
    let shown = target.escape_debug();
    let unfollowed = format!("{name} is a symlink to {shown}, which was not followed.\n");
    if !plain(&target) {
        return Ok(unfollowed);
    }
    match git.tree_entry(repo, base, &target).await? {
        Some(next) if matches!(next.mode.as_str(), "100644" | "100755") => {
            let note = format!(
                "{name} is a symlink to {target} at the merge-base; the content of {target} follows.\n"
            );
            content(git, repo, (name, &next), &note).await
        }
        _ => Ok(unfollowed),
    }
}

async fn text<G: Git>(git: &G, at: (&Path, &str), name: &str) -> Result<String, Error> {
    let Some(entry) = git.tree_entry(at.0, at.1, name).await? else {
        return Ok(format!("{name} is not present at the merge-base.\n"));
    };
    match entry.mode.as_str() {
        "100644" | "100755" => content(git, at.0, (name, &entry), "").await,
        "120000" => symlink(git, at, (name, &entry)).await,
        _ => Ok(format!("{name} is not a regular file at the merge-base.\n")),
    }
}

pub(super) async fn write<G: Git>(
    git: &G,
    at: (&Path, &str),
    dir: &Path,
    warnings: &mut Vec<String>,
) -> Result<(), Error> {
    let rules = dir.join("rules");
    let made = create_private_dir_async(rules.clone()).await;
    made.map_err(Error::io("create rules directory"))?;
    for name in FILES {
        let body = match text(git, at, name).await {
            Ok(body) => body,
            Err(error) => format!("{name} is unavailable: {}\n", reason(&error)),
        };
        note_truncation(warnings, &format!("rules/{name}"), &body);
        put(&rules.join(name), body).await?;
    }
    Ok(())
}
