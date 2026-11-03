use super::signals::Signals;
use super::wait::{self, Waited};
use crate::app::AppConfig;
use crate::error::Error;
use crate::git::Git;
use crate::pr_url::PrRef;
use crate::store::jobs::{EnqueueRequest, EnqueuedJob};
use crate::store::{Store, records::JobOrigin};
use crate::{recovery, render};

pub(super) async fn for_job<G: Git>(
    config: &AppConfig,
    store: &Store,
    git: &G,
    (pr, force): (&PrRef, bool),
    job: &mut EnqueuedJob,
    signals: &mut Signals,
) -> Result<Waited, Error> {
    loop {
        let waited = wait::until_lock_or_end(
            &config.paths.lock(),
            store,
            job.job_id,
            &pr.label(),
            signals,
        )
        .await?;
        let Waited::Locked(lock) = waited else {
            return Ok(waited);
        };
        lock.announce(crate::lock::Role::Review).await?;
        let report = recovery::run(&config.paths, store, git, &config.settings).await?;
        for line in report.lines() {
            eprintln!("{}", render::sanitize(&line));
        }
        if report.manual_crashes.contains(&job.job_id) {
            let request = EnqueueRequest {
                origin: JobOrigin::Manual,
                force,
            };
            *job = store
                .enqueue_review(pr, &config.process, request)
                .await?
                .job;
        }
        if store.stored_result(job.job_id).await?.state != "queued" {
            return Ok(Waited::Finished);
        }
        if !store
            .job_delayed(job.job_id, crate::store::now_ms())
            .await?
        {
            return Ok(Waited::Locked(lock));
        }
    }
}
