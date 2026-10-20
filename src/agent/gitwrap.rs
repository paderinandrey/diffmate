use crate::error::Error;
use crate::paths::create_private_dir_async;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[cfg(test)]
mod tests;

pub(crate) fn find_git(path: &OsStr) -> Option<PathBuf> {
    let mut candidates = std::env::split_paths(path).map(|dir| dir.join("git"));
    candidates.find(|candidate| candidate.is_file())
}

fn script(real_git: &Path) -> String {
    let real = real_git.to_string_lossy().replace('\'', "'\\''");
    format!(
        "#!/bin/sh\nGIT_CONFIG_GLOBAL=/dev/null\nGIT_CONFIG_NOSYSTEM=1\nexport GIT_CONFIG_GLOBAL GIT_CONFIG_NOSYSTEM\nexec '{real}' \"$@\"\n"
    )
}

pub(crate) async fn install(prompt_file: &Path, real_git: &Path) -> Result<PathBuf, Error> {
    let dir = prompt_file.parent().unwrap_or(Path::new(".")).join("bin");
    let make = create_private_dir_async(dir.clone()).await;
    make.map_err(Error::io("create agent bin directory"))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o700);
    let what = "write git wrapper";
    let mut file = options
        .open(dir.join("git"))
        .await
        .map_err(Error::io(what))?;
    file.write_all(script(real_git).as_bytes())
        .await
        .map_err(Error::io(what))?;
    file.flush().await.map_err(Error::io(what))?;
    Ok(dir)
}
