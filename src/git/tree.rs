use super::GitCli;
use crate::error::Error;
use crate::proc::{Limits, excerpt, finish};
use std::path::Path;
use std::process::Stdio;

pub(crate) struct TreeEntry {
    pub mode: String,
    pub oid: String,
    pub size: u64,
}

fn parse(line: &str) -> Option<TreeEntry> {
    let (fields, _) = line.split_once('\t')?;
    let mut fields = fields.split_whitespace();
    let (mode, _kind, oid, size) = (
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
    );
    Some(TreeEntry {
        mode: mode.into(),
        oid: oid.into(),
        size: size.parse().unwrap_or(0),
    })
}

pub(super) async fn entry(
    git: &GitCli,
    repo: &Path,
    commit: &str,
    path: &str,
) -> Result<Option<TreeEntry>, Error> {
    let args = ["ls-tree", "--long", commit, "--", path];
    let listing = GitCli::text(git.command(Some(repo), args)).await?;
    Ok(listing.lines().next().and_then(parse))
}

pub(super) async fn blob(
    git: &GitCli,
    repo: &Path,
    oid: &str,
    cap: usize,
) -> Result<Vec<u8>, Error> {
    let mut command = git.command(Some(repo), ["cat-file", "blob", oid]);
    command.stdin(Stdio::null()).stdout(Stdio::piped());
    command.stderr(Stdio::piped()).kill_on_drop(true);
    let child = command.spawn().map_err(Error::io("start git"))?;
    let limits = Limits {
        stdout: cap,
        stderr: 64 << 10,
    };
    let done = Box::pin(finish(child, None, limits)).await;
    let done = done.map_err(Error::io("run git"))?;
    if done.status.success() {
        return Ok(done.stdout);
    }
    Err(Error::Command {
        program: "git",
        args: format!("cat-file blob {oid}"),
        detail: format!("{}: {}", done.status, excerpt(&done.stderr, 500).trim()),
    })
}
