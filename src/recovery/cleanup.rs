use super::{RecoveryReport, scan};
use crate::error::Error;
use crate::git::Git;
use crate::paths::Paths;
use crate::store::Store;

pub(super) async fn clean_runs<G: Git>(
    paths: &Paths,
    store: &Store,
    git: &G,
    report: &mut RecoveryReport,
) -> Result<(), Error> {
    let found = scan::run_dirs(&paths.runs_dir()).await?;
    for name in &found.others {
        report
            .warnings
            .push(format!("left {name} in the runs directory: not a run"));
    }
    let locations = store.run_locations(found.runs.clone()).await?;
    for run in found.runs {
        let location = locations.get(&run);
        if location.is_some_and(|known| known.running) {
            continue;
        }
        let worktree = paths.worktree(run);
        let present = matches!(tokio::fs::try_exists(&worktree).await, Ok(true));
        if let (Some(known), true) = (location, present) {
            let repo = paths.repo_dir(&known.owner, &known.repo);
            if let Err(error) = git.worktree_remove(&repo, &worktree).await {
                report.warnings.push(format!("run {run}: {error}"));
            }
        }
        match tokio::fs::remove_dir_all(paths.run_dir(run)).await {
            Ok(()) => report.removed_run_dirs.push(run),
            Err(error) => report
                .warnings
                .push(format!("run {run}: remove directory: {error}")),
        }
    }
    Ok(())
}
