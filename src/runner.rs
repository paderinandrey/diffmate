mod agent_run;
mod cancelled;
mod cleanup;
mod outcome;
mod process;
mod resolve;
mod result;
mod steps;
#[cfg(test)]
mod tests;

use crate::agent::{Agent, AgentOutcome};
use crate::cancel::{self, CancelRx};
use crate::config::Settings;
use crate::gh::{Gh, PrMeta};
use crate::git::Git;
use crate::normalize::{Normalized, normalize};
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::Store;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::Revision;
use cancelled::cancelled_before_run;
use result::job_failed;

pub(crate) use result::{JobEnd, ReviewResult};

pub(crate) struct Deps<G, H, A> {
    pub git: G,
    pub gh: H,
    pub agent: A,
    pub settings: Settings,
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
        Ok(()) => cancel::raced(&mut stop, resolve::resolve(paths, store, deps, pr, job)).await,
        Err(error) => Some(Err(error)),
    };
    match resolved {
        None => cancelled_before_run(store, job, &deps.settings).await,
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
            "PR head differs from reviewed revision: gh reported {seen}, reviewed {reviewed}"
        ));
    }
    let normalized = match &outcome {
        AgentOutcome::Succeeded { output, .. } => {
            normalize(output, &prepared.index, &prepared.previous_ids)
        }
        AgentOutcome::Failed { .. } | AgentOutcome::Cancelled { .. } => Normalized::default(),
    };
    outcome::store(
        store,
        (job, run, revision),
        (outcome, normalized),
        warnings,
        &deps.settings,
    )
    .await
}
