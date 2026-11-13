use super::Check;
use crate::app::AppConfig;
use std::fs::Metadata;
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

pub(super) async fn check(config: &AppConfig) -> Check {
    let state = config.paths.state_dir().to_path_buf();
    let cache = config.paths.cache_dir().to_path_buf();
    let file = config.config_file.clone();
    let reported = tokio::task::spawn_blocking(move || problems(&state, &cache, &file)).await;
    match reported {
        Ok(problems) if problems.is_empty() => Check::pass("storage: ok"),
        Ok(problems) => Check::fail(format!("storage: {}", problems.join(" "))),
        Err(error) => Check::fail(format!("storage: {error}")),
    }
}

fn problems(state: &Path, cache: &Path, file: &Path) -> Vec<String> {
    let mut found = Vec::new();
    if let Some(problem) = directory(state) {
        found.push(problem);
    }
    if let Some(problem) = directory(cache) {
        found.push(problem);
    }
    if let Err(error) = crate::config::load(file) {
        found.push(error.to_string());
    }
    found
}

fn directory(path: &Path) -> Option<String> {
    match std::fs::metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => Some(format!("{}: {error}", path.display())),
        Ok(meta) if !meta.is_dir() => Some(aside(path)),
        Ok(meta) => mode_owner(path, &meta),
    }
}

fn aside(path: &Path) -> String {
    format!(
        "{} is not a directory; move it aside so diffmate review or diffmate watch can create the directory",
        path.display()
    )
}

fn mode_owner(path: &Path, meta: &Metadata) -> Option<String> {
    let mode = meta.permissions().mode() & 0o777;
    let owned = meta.uid() == rustix::process::getuid().as_raw();
    (mode != 0o700 || !owned).then(|| format!("{}: chmod 700", path.display()))
}
