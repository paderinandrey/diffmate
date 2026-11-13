use std::ffi::OsStr;
use std::path::PathBuf;

pub(super) async fn which(path: &OsStr, name: &str) -> Option<PathBuf> {
    let path = path.to_os_string();
    let name = name.to_owned();
    let found = tokio::task::spawn_blocking(move || find(&path, &name)).await;
    found.ok().flatten()
}

fn find(path: &OsStr, name: &str) -> Option<PathBuf> {
    let mut candidates = std::env::split_paths(path).map(|dir| dir.join(name));
    candidates.find(|candidate| candidate.is_file())
}
