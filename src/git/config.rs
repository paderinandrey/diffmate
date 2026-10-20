use super::{GitCli, os};
use crate::error::Error;
use std::path::Path;

async fn ensure_empty(dir: &Path) -> Result<(), Error> {
    let read = Error::io("read hooks directory");
    let mut entries = tokio::fs::read_dir(dir).await.map_err(read)?;
    let read = Error::io("read hooks directory");
    match entries.next_entry().await.map_err(read)? {
        None => Ok(()),
        Some(_) => Err(Error::HooksNotEmpty(dir.to_path_buf())),
    }
}

pub(super) async fn write(
    git: &GitCli,
    repo: &Path,
    remote_url: &str,
    hooks_dir: &Path,
) -> Result<(), Error> {
    ensure_empty(hooks_dir).await?;
    let settings = [
        ("core.hooksPath", os(hooks_dir)),
        ("core.fsmonitor", os("false")),
        ("core.symlinks", os("false")),
        ("gc.autoDetach", os("false")),
        ("maintenance.autoDetach", os("false")),
        ("remote.origin.url", os(remote_url)),
    ];
    for (key, value) in settings {
        let args = [os("config"), os("--local"), os(key), value];
        GitCli::run(git.command(Some(repo), args)).await?;
    }
    let list = ["config", "--local", "--name-only", "--list"];
    let keys = GitCli::text(git.command(Some(repo), list)).await?;
    if keys
        .lines()
        .any(|key| key.eq_ignore_ascii_case("remote.origin.fetch"))
    {
        let unset = ["config", "--local", "--unset-all", "remote.origin.fetch"];
        GitCli::run(git.command(Some(repo), unset)).await?;
    }
    Ok(())
}
