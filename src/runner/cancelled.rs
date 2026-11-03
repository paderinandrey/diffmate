use super::{JobEnd, ReviewResult, job_failed};
use crate::config::Settings;
use crate::error::Error;
use crate::recovery::rules::{Failure, JobNext, after_failure};
use crate::store::{Store, jobs::EnqueuedJob, now_ms, records::JobOrigin};

async fn record(store: &Store, job: &EnqueuedJob, settings: &Settings) -> Result<JobEnd, Error> {
    let stop = store.stop_reason(job.job_id).await?;
    let (origin, state) = store.job_retry_state(job.job_id).await?;
    let mut next = after_failure(
        origin,
        &Failure::from_stop(stop),
        state,
        settings.max_retries(),
        now_ms(),
    );
    if let JobNext::Fail { reason, .. } = &mut next
        && origin == JobOrigin::Manual
    {
        *reason = "interrupted before the run started".into();
        if let Some(cause) = stop {
            *reason = format!("{reason} ({})", cause.as_str());
        }
    }
    store.resolve_job(job.job_id, &next).await?;
    Ok(JobEnd::from(&next))
}

pub(super) async fn cancelled_before_run(
    store: &Store,
    job: &EnqueuedJob,
    settings: &Settings,
) -> ReviewResult {
    match record(store, job, settings).await {
        Ok(job_end) => ReviewResult::Cancelled {
            run_id: None,
            job_end,
        },
        Err(error) => job_failed(store, job, format!("recording cancellation: {error}")).await,
    }
}
