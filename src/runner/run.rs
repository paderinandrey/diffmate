use super::{Deps, ReviewResult, agent_run, cleanup, job_failed, outcome};
use crate::agent::{Agent, AgentOutcome};
use crate::cancel::{self, CancelRx};
use crate::gh::{Gh, PrMeta};
use crate::git::Git;
use crate::normalize::{Normalized, normalize};
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::{Store, jobs::EnqueuedJob, records::Revision};

pub(super) async fn review_run<G: Git, H: Gh, A: Agent>(
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
    if matches!(outcome, AgentOutcome::Succeeded { .. }) {
        let mut stop = cancel.clone();
        match cancel::raced(&mut stop, deps.gh.pr_view(pr)).await {
            Some(Ok(latest)) => {
                if let Err(error) = store.set_known_head(job.target_id, &latest.head_sha).await {
                    return job_failed(store, job, error.to_string()).await;
                }
            }
            Some(Err(error)) => warnings.push(format!("could not re-check the PR head: {error}")),
            None => {}
        }
    }
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
