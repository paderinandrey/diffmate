use super::{ReviewResult, job_failed};
use crate::agent::{AgentOutcome, FailureDetail, FailureKind};
use crate::cancel::StopReason;
use crate::config::Settings;
use crate::error::Error;
use crate::normalize::Normalized;
use crate::recovery::rules::{Failure, JobNext, after_failure};
use crate::store::jobs::EnqueuedJob;
use crate::store::records::Revision;
use crate::store::{RunId, Store};
use failed::failed;

mod failed;
mod succeeded;
#[cfg(test)]
mod tests;

pub(super) async fn store(
    store: &Store,
    (job, run, _revision): (&EnqueuedJob, RunId, &Revision),
    (outcome, normalized): (AgentOutcome, Normalized),
    cleanup_warnings: Vec<String>,
    settings: &Settings,
) -> ReviewResult {
    match outcome {
        AgentOutcome::Succeeded {
            output,
            raw,
            metrics,
        } => {
            succeeded::store(
                store,
                (job, run),
                (output, raw, metrics),
                normalized,
                cleanup_warnings,
            )
            .await
        }
        AgentOutcome::Cancelled { metrics } => {
            let stop = store.stop_reason(job.job_id).await.unwrap_or(None);
            let (kind, message) = match stop {
                Some(StopReason::Timeout) => {
                    (FailureKind::Timeout, "stopped by the run timeout".into())
                }
                Some(cause) => (
                    FailureKind::Interrupted,
                    format!("stopped: interrupted ({})", cause.as_str()),
                ),
                None => (FailureKind::Interrupted, "interrupted by signal".into()),
            };
            let detail = FailureDetail {
                message,
                ..FailureDetail::default()
            };
            failed(
                store,
                (job, run),
                (kind, detail, metrics),
                stop,
                cleanup_warnings,
                settings,
            )
            .await
        }
        AgentOutcome::Failed {
            kind,
            detail,
            metrics,
        } => {
            failed(
                store,
                (job, run),
                (kind, detail, metrics),
                None,
                cleanup_warnings,
                settings,
            )
            .await
        }
    }
}

pub(super) async fn next(
    store: &Store,
    job: &EnqueuedJob,
    failure: &Failure,
    settings: &Settings,
) -> Result<JobNext, Error> {
    let (origin, state) = store.job_retry_state(job.job_id).await?;
    Ok(after_failure(
        origin,
        failure,
        state,
        settings.max_retries(),
        crate::store::now_ms(),
    ))
}
