use crate::agent::FailureKind;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::FindingRecord;
use crate::store::{RunId, Store};

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
    Cancelled {
        run_id: Option<RunId>,
    },
}

pub(super) async fn job_failed(store: &Store, job: &EnqueuedJob, reason: String) -> ReviewResult {
    let reason = match store.fail_job(job.job_id, reason.clone()).await {
        Ok(()) => reason,
        Err(error) => format!("{reason}; recording the failure also failed: {error}"),
    };
    ReviewResult::JobFailed { reason }
}

pub(super) async fn cancelled_before_run(store: &Store, job: &EnqueuedJob) -> ReviewResult {
    let reason = "interrupted before the run started".to_string();
    match store.fail_job(job.job_id, reason.clone()).await {
        Ok(()) => ReviewResult::Cancelled { run_id: None },
        Err(error) => ReviewResult::JobFailed {
            reason: format!("{reason}; recording it failed: {error}"),
        },
    }
}
