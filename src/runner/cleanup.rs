use crate::git::Git;
use crate::paths::Paths;
use crate::store::RunId;
use std::io::ErrorKind;
use std::path::Path;

pub(super) async fn run<G: Git>(git: &G, paths: &Paths, repo: &Path, run: RunId) -> Vec<String> {
    let mut problems = Vec::new();
    let worktree = paths.worktree(run);
    let present = matches!(tokio::fs::try_exists(&worktree).await, Ok(true));
    let removed = if present {
        git.worktree_remove(repo, &worktree).await
    } else {
        Ok(())
    };
    if let Err(error) = removed {
        problems.push(format!("cleanup: {error}"));
    }
    if let Err(error) = git.worktree_prune(repo).await {
        problems.push(format!("cleanup: {error}"));
    }
    let removed = tokio::fs::remove_dir_all(paths.run_dir(run)).await;
    if let Err(error) = removed.or_else(|e| {
        if e.kind() == ErrorKind::NotFound {
            Ok(())
        } else {
            Err(e)
        }
    }) {
        problems.push(format!("cleanup: {error}"));
    }
    problems
}
