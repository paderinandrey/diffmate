#[cfg(test)]
mod fake;
mod scan;
#[cfg(test)]
mod tests;

use crate::error::Error;
use crate::git::Git;
use crate::paths::Paths;
use crate::store::{JobId, RunId, Store};

#[derive(Default)]
pub(crate) struct RecoveryReport {
    pub interrupted_runs: Vec<RunId>,
    pub failed_jobs: Vec<JobId>,
    pub removed_run_dirs: Vec<RunId>,
    pub warnings: Vec<String>,
}

impl RecoveryReport {
    pub fn lines(&self) -> Vec<String> {
        let runs = self.interrupted_runs.iter();
        let marked =
            runs.map(|run| format!("recovery: run {run} marked failed (interrupted: crash)"));
        let jobs = self.failed_jobs.iter();
        let jobs = jobs.map(|job| format!("recovery: job {job} failed (interrupted: crash)"));
        let removed = self.removed_run_dirs.iter();
        let removed = removed.map(|run| format!("recovery: removed run directory {run}"));
        let warnings = self.warnings.iter();
        let warnings = warnings.map(|warning| format!("recovery: warning: {warning}"));
        marked.chain(jobs).chain(removed).chain(warnings).collect()
    }
}

async fn clean_runs<G: Git>(
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

pub(crate) async fn run<G: Git>(
    paths: &Paths,
    store: &Store,
    git: &G,
) -> Result<RecoveryReport, Error> {
    let stopped = store.stop_running().await?;
    let mut report = RecoveryReport {
        interrupted_runs: stopped.runs,
        failed_jobs: stopped.jobs,
        ..RecoveryReport::default()
    };
    clean_runs(paths, store, git, &mut report).await?;
    for clone in scan::clones(&paths.repos_dir()).await? {
        if let Err(error) = git.worktree_prune(&clone).await {
            report
                .warnings
                .push(format!("prune {}: {error}", clone.display()));
        }
    }
    Ok(report)
}
