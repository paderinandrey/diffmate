mod cleanup;
mod outcome;
mod resolve;
mod steps;

use crate::agent::{Agent, AgentOutcome, FailureKind};
use crate::gh::{Gh, PrMeta};
use crate::git::Git;
use crate::normalize::{Normalized, normalize};
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::{FindingRecord, Revision};
use crate::store::{RunId, Store};

pub(crate) struct Deps<G, H, A> {
    pub git: G,
    pub gh: H,
    pub agent: A,
}

#[expect(dead_code)]
pub(crate) enum ReviewResult {
    Succeeded {
        run_id: RunId,
        head_sha: String,
        findings: Vec<FindingRecord>,
        limitation: Option<String>,
        future_improvements: Vec<String>,
        warnings: Vec<String>,
    },
    RunFailed {
        run_id: RunId,
        kind: FailureKind,
        message: String,
    },
    JobFailed {
        reason: String,
    },
}

async fn job_failed(store: &Store, job: &EnqueuedJob, reason: String) -> ReviewResult {
    let reason = match store.fail_job(job.job_id, reason.clone()).await {
        Ok(()) => reason,
        Err(error) => format!("{reason}; recording the failure also failed: {error}"),
    };
    ReviewResult::JobFailed { reason }
}

pub(crate) async fn execute_review_job<G: Git, H: Gh, A: Agent>(
    paths: &Paths,
    store: &Store,
    deps: &Deps<G, H, A>,
    pr: &PrRef,
    job: &EnqueuedJob,
) -> ReviewResult {
    let resolved = match store.start_job(job.job_id).await {
        Ok(()) => resolve::resolve(paths, deps, pr).await,
        Err(error) => Err(error),
    };
    match resolved {
        Ok((meta, revision)) => review_run(paths, store, deps, (pr, job), (&meta, &revision)).await,
        Err(error) => job_failed(store, job, error.to_string()).await,
    }
}

async fn review_run<G: Git, H: Gh, A: Agent>(
    paths: &Paths,
    store: &Store,
    deps: &Deps<G, H, A>,
    (pr, job): (&PrRef, &EnqueuedJob),
    (meta, revision): (&PrMeta, &Revision),
) -> ReviewResult {
    let run = match store.insert_running_run(job, revision).await {
        Ok(run) => run,
        Err(error) => return job_failed(store, job, error.to_string()).await,
    };
    let ids = (pr, job.target_id, run);
    let (outcome, mut prepared) = steps::run_agent(paths, store, deps, ids, (meta, revision)).await;
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
        AgentOutcome::Failed { .. } => Normalized::default(),
    };
    outcome::store(store, (job, run, revision), (outcome, normalized), warnings).await
}
