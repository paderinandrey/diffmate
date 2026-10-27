mod agent_run;
mod cleanup;
mod outcome;
mod resolve;
mod result;
mod steps;
#[cfg(test)]
mod tests;

use crate::agent::{Agent, AgentOutcome};
use crate::cancel::{self, CancelRx};
use crate::gh::{Gh, PrMeta};
use crate::git::Git;
use crate::normalize::{Normalized, normalize};
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::Revision;
use crate::store::{RunId, Store};
use result::{cancelled_before_run, job_failed};
use tokio::sync::mpsc::UnboundedReceiver;

pub(crate) use result::ReviewResult;

async fn record_agent_process(
    store: &Store,
    run: RunId,
    mut pids: UnboundedReceiver<u32>,
) -> Vec<String> {
    let mut problems = Vec::new();
    while let Some(pid) = pids.recv().await {
        if let Err(error) = store.set_agent_process(run, pid, pid).await {
            problems.push(format!("recording the agent process failed: {error}"));
        }
    }
    problems
}

pub(crate) struct Deps<G, H, A> {
    pub git: G,
    pub gh: H,
    pub agent: A,
}

pub(crate) async fn execute_review_job<G: Git, H: Gh, A: Agent>(
    paths: &Paths,
    store: &Store,
    deps: &Deps<G, H, A>,
    pr: &PrRef,
    job: &EnqueuedJob,
    cancel: &CancelRx,
) -> ReviewResult {
    let mut stop = cancel.clone();
    let resolved = match store.start_job(job.job_id).await {
        Ok(()) => cancel::raced(&mut stop, resolve::resolve(paths, deps, pr)).await,
        Err(error) => Some(Err(error)),
    };
    match resolved {
        None => cancelled_before_run(store, job).await,
        Some(Ok((meta, revision))) => {
            let revisions = (&meta, &revision);
            review_run(paths, store, deps, (pr, job), revisions, cancel).await
        }
        Some(Err(error)) => job_failed(store, job, error.to_string()).await,
    }
}

async fn review_run<G: Git, H: Gh, A: Agent>(
    paths: &Paths,
    store: &Store,
    deps: &Deps<G, H, A>,
    (pr, job): (&PrRef, &EnqueuedJob),
    (meta, revision): (&PrMeta, &Revision),
    cancel: &CancelRx,
) -> ReviewResult {
    let run = match store.insert_running_run(job, revision).await {
        Ok(run) => run,
        Err(error) => return job_failed(store, job, error.to_string()).await,
    };
    let ids = (pr, job.target_id, run);
    let ran = agent_run::run_agent(paths, store, deps, ids, (meta, revision), cancel);
    let (outcome, mut prepared) = ran.await;
    let repo = paths.repo_dir(&pr.owner, &pr.repo);
    let mut warnings = std::mem::take(&mut prepared.warnings);
    warnings.extend(cleanup::run(&deps.git, paths, &repo, run).await);
    if meta.head_sha != revision.head_sha {
        let (seen, reviewed) = (&meta.head_sha, &revision.head_sha);
        warnings.push(format!(
            "PR head moved during preparation: gh reported {seen}, reviewed {reviewed}"
        ));
    }
    let normalized = match &outcome {
        AgentOutcome::Succeeded { output, .. } => {
            normalize(output, &prepared.index, &prepared.previous_ids)
        }
        AgentOutcome::Failed { .. } | AgentOutcome::Cancelled { .. } => Normalized::default(),
    };
    outcome::store(store, (job, run, revision), (outcome, normalized), warnings).await
}
