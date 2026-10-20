use crate::error::Error;
use crate::store::RunId;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(super) struct RunsDir {
    pub runs: Vec<RunId>,
    pub others: Vec<String>,
}

fn read_error(error: std::io::Error) -> Error {
    Error::io("read a recovery directory")(error)
}

async fn entries(dir: &Path) -> Result<Vec<(String, PathBuf, bool)>, Error> {
    let mut found = Vec::new();
    let mut reader = match tokio::fs::read_dir(dir).await {
        Ok(reader) => reader,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(found),
        Err(error) => return Err(read_error(error)),
    };
    while let Some(entry) = reader.next_entry().await.map_err(read_error)? {
        let is_dir = entry.file_type().await.map_err(read_error)?.is_dir();
        let name = entry.file_name().to_string_lossy().into_owned();
        found.push((name, entry.path(), is_dir));
    }
    Ok(found)
}

pub(super) async fn run_dirs(dir: &Path) -> Result<RunsDir, Error> {
    let mut found = RunsDir {
        runs: Vec::new(),
        others: Vec::new(),
    };
    for (name, _, is_dir) in entries(dir).await? {
        let digits = !name.is_empty() && name.bytes().all(|b| b.is_ascii_digit());
        match name.parse::<RunId>() {
            Ok(run) if digits && is_dir => found.runs.push(run),
            _ => found.others.push(name),
        }
    }
    found.runs.sort_unstable();
    found.others.sort();
    Ok(found)
}

pub(super) async fn clones(repos: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut found = Vec::new();
    for (_, owner, is_dir) in entries(repos).await? {
        if !is_dir {
            continue;
        }
        for (_, path, is_dir) in entries(&owner).await? {
            if is_dir && path.extension().is_some_and(|extension| extension == "git") {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}
