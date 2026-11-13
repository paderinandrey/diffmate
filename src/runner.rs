mod agent_run;
mod cancelled;
mod cleanup;
mod outcome;
mod process;
mod resolve;
mod result;
mod run;
mod steps;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use agent_run::fakes::FakeGit;

use crate::agent::Agent;
use crate::cancel::{self, CancelRx};
use crate::config::Settings;
use crate::crash::crash_point;
use crate::gh::Gh;
use crate::git::Git;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::Store;
use crate::store::jobs::EnqueuedJob;
use cancelled::cancelled_before_run;
use result::job_failed;

pub(crate) use result::{Follow, JobEnd, ReviewResult};

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
    let started = match store.start_job(job.job_id).await {
        Ok(started) => started,
        Err(error) => return job_failed(store, job, error.to_string()).await,
    };
    crash_point!("job-started");
    let resolved = cancel::raced(&mut stop, resolve::resolve(paths, store, deps, pr, job)).await;
    match resolved {
        None => cancelled_before_run(store, job, &deps.settings).await,
        Some(Ok((meta, revision))) => {
            crash_point!("revision");
            if let Err(error) = store.set_known_head(job.target_id, &meta.head_sha).await {
                return job_failed(store, job, error.to_string()).await;
            }
            if !started.force {
                match store
                    .succeeded_run_for_head(job.target_id, &revision.head_sha)
                    .await
                {
                    Ok(Some(run)) => return result::reuse(store, job, run).await,
                    Ok(None) => {}
                    Err(error) => return job_failed(store, job, error.to_string()).await,
                }
            }
            let revisions = (&meta, &revision);
            run::review_run(paths, store, deps, (pr, job), revisions, cancel).await
        }
        Some(Err(error)) => job_failed(store, job, error.to_string()).await,
    }
}
