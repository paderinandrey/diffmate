use super::Deps;
use crate::crash::crash_point;
use crate::error::Error;
use crate::gh::{Gh, PrMeta};
use crate::git::Git;
use crate::paths::Paths;
use crate::pr_url::PrRef;
use crate::store::Store;
use crate::store::jobs::{EnqueuedJob, StartedJob};
use crate::store::poll::Admission;
use crate::store::records::{JobOrigin, Revision};

#[cfg(test)]
mod tests;

pub(super) async fn resolve<G: Git, H: Gh, A>(
    paths: &Paths,
    store: &Store,
    deps: &Deps<G, H, A>,
    pr: &PrRef,
    job: &EnqueuedJob,
    started: &StartedJob,
) -> Result<Option<(PrMeta, Revision)>, Error> {
    let meta = crate::gh::verified_view(&deps.gh, pr).await?;
    let automatic = started.origin == JobOrigin::Automatic;
    let closed = super::run::closed(&meta.state);
    store
        .observe_review_head(job.target_id, &meta.head_sha, closed)
        .await?;
    if automatic && (closed || meta.is_draft) {
        let admission = if closed {
            Admission::Ready
        } else {
            Admission::Draft
        };
        store.admit_automatic(job, admission).await?;
        return Ok(None);
    }
    let repo = paths.repo_dir(&pr.owner, &pr.repo);
    deps.git
        .ensure_clone(&repo, &pr.remote_url(), &paths.empty_hooks_dir())
        .await?;
    if automatic && let Some(revision) = store.retry_revision(job.job_id).await? {
        return Ok(store
            .admit_automatic(job, Admission::Ready)
            .await?
            .then_some((meta, revision)));
    }
    deps.git.fetch_pr(&repo, pr.number, &meta.base_ref).await?;
    crash_point!("fetched");
    let head = deps
        .git
        .rev_parse(&repo, &format!("refs/diffmate/pull/{}/head", pr.number))
        .await?;
    if automatic {
        if started
            .head_sha
            .as_ref()
            .is_some_and(|queued| *queued != head)
        {
            store
                .admit_automatic(
                    job,
                    Admission::NewHead {
                        head,
                        debounce: deps.settings.debounce(),
                        force: started.force,
                    },
                )
                .await?;
            return Ok(None);
        }
        if !store.admit_automatic(job, Admission::Ready).await? {
            return Ok(None);
        }
    }
    let base_ref = &meta.base_ref;
    let tip = deps
        .git
        .rev_parse(&repo, &format!("refs/diffmate/base/{base_ref}"))
        .await?;
    let merge_base_sha = deps.git.merge_base(&repo, &tip, &head).await?;
    let revision = Revision {
        head_sha: head,
        merge_base_sha,
        base_tip_sha: tip,
        base_ref: base_ref.clone(),
    };
    Ok(Some((meta, revision)))
}
