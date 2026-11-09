use super::{ReviewResult, job_failed};
use crate::agent::{FailureDetail, FailureKind};
use crate::cancel::StopReason;
use crate::config::Settings;
use crate::crash::crash_point;
use crate::recovery::rules::{Failure, is_transient};
use crate::runner::JobEnd;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::Metrics;
use crate::store::{RunId, Store};

pub(super) async fn failed(
    store: &Store,
    (job, run): (&EnqueuedJob, RunId),
    (kind, detail, metrics): (FailureKind, FailureDetail, Metrics),
    stop: Option<StopReason>,
    cleanup_warnings: Vec<String>,
    settings: &Settings,
) -> ReviewResult {
    let failure = match kind {
        FailureKind::Timeout => Failure::Timeout,
        FailureKind::Interrupted => Failure::from_stop(stop),
        FailureKind::AgentError if is_transient(&detail) => Failure::TransientAgent,
        other => Failure::Terminal(other),
    };
    let next = match super::next(store, job, &failure, settings).await {
        Ok(next) => next,
        Err(error) => return job_failed(store, job, format!("reading retry state: {error}")).await,
    };
    let job_end = JobEnd::from(&next);
    let mut message = detail.message.clone();
    for warning in cleanup_warnings {
        message = format!("{message}; {warning}");
    }
    match store
        .finish_failed(run, job.job_id, (kind, stop), detail, metrics, &next)
        .await
    {
        Ok(()) => {
            crash_point!("stored");
            if matches!(kind, FailureKind::Interrupted) {
                ReviewResult::Cancelled {
                    run_id: Some(run),
                    job_end,
                }
            } else {
                ReviewResult::RunFailed {
                    run_id: run,
                    kind,
                    message,
                    job_end,
                }
            }
        }
        Err(error) => job_failed(store, job, format!("storing run {run}: {error}")).await,
    }
}
