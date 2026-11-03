mod cleanup;
#[cfg(test)]
mod fake;
pub(crate) mod rules;
mod scan;
#[cfg(test)]
mod tests;

use crate::config::Settings;
use crate::error::Error;
use crate::git::Git;
use crate::paths::Paths;
use crate::store::{JobId, RunId, Store};

#[derive(Default)]
pub(crate) struct RecoveryReport {
    pub runs: Vec<(RunId, String)>,
    pub jobs: Vec<(JobId, String)>,
    pub manual_crashes: Vec<JobId>,
    pub removed_run_dirs: Vec<RunId>,
    pub warnings: Vec<String>,
}

impl RecoveryReport {
    pub fn lines(&self) -> Vec<String> {
        let marked = self
            .runs
            .iter()
            .map(|(run, text)| format!("recovery: run {run} marked failed ({text})"));
        let jobs = self
            .jobs
            .iter()
            .map(|(job, text)| format!("recovery: job {job} {text}"));
        let removed = self.removed_run_dirs.iter();
        let removed = removed.map(|run| format!("recovery: removed run directory {run}"));
        let warnings = self.warnings.iter();
        let warnings = warnings.map(|warning| format!("recovery: warning: {warning}"));
        marked.chain(jobs).chain(removed).chain(warnings).collect()
    }
}

pub(crate) async fn run<G: Git>(
    paths: &Paths,
    store: &Store,
    git: &G,
    settings: &Settings,
) -> Result<RecoveryReport, Error> {
    let stopped = store
        .resolve_running(settings.max_retries(), crate::store::now_ms())
        .await?;
    let mut report = RecoveryReport {
        runs: stopped.runs,
        jobs: stopped.jobs,
        manual_crashes: stopped.manual_crashes,
        ..RecoveryReport::default()
    };
    cleanup::clean_runs(paths, store, git, &mut report).await?;
    for clone in scan::clones(&paths.repos_dir()).await? {
        if let Err(error) = git.worktree_prune(&clone).await {
            report
                .warnings
                .push(format!("prune {}: {error}", clone.display()));
        }
    }
    Ok(report)
}
