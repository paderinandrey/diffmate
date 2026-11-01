use super::{ReviewResult, job_failed};
use crate::agent::{FailureDetail, FailureKind};
use crate::cancel::StopReason;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::Metrics;
use crate::store::{RunId, Store};

pub(super) async fn failed(
    store: &Store,
    (job, run): (&EnqueuedJob, RunId),
    (kind, detail, metrics): (FailureKind, FailureDetail, Metrics),
    stop: Option<StopReason>,
    cleanup_warnings: Vec<String>,
) -> ReviewResult {
    let mut message = detail.message.clone();
    for warning in cleanup_warnings {
        message = format!("{message}; {warning}");
    }
    match store
        .finish_failed(run, job.job_id, kind, stop, detail, metrics)
        .await
    {
        Ok(()) if matches!(kind, FailureKind::Interrupted) => {
            ReviewResult::Cancelled { run_id: Some(run) }
        }
        Ok(()) => ReviewResult::RunFailed {
            run_id: run,
            kind,
            message,
        },
        Err(error) => job_failed(store, job, format!("storing run {run}: {error}")).await,
    }
}
