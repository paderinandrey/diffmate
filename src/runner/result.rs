use crate::agent::FailureKind;
use crate::crash::crash_point;
use crate::store::jobs::EnqueuedJob;
use crate::store::records::FindingRecord;
use crate::store::{RunId, Store};

mod stored;

#[expect(dead_code)]
pub(crate) enum ReviewResult {
    Succeeded {
        reused: bool,
        run_id: RunId,
        head_sha: String,
        known_head: Option<String>,
        findings: Vec<FindingRecord>,
        limitation: Option<String>,
        future_improvements: Vec<String>,
        warnings: Vec<String>,
    },
    RunFailed {
        run_id: RunId,
        kind: FailureKind,
        message: String,
        job_end: JobEnd,
    },
    JobFailed {
        reason: String,
    },
    Cancelled {
        run_id: Option<RunId>,
        job_end: JobEnd,
    },
}

pub(super) async fn job_failed(store: &Store, job: &EnqueuedJob, reason: String) -> ReviewResult {
    let reason = match store.fail_job(job.job_id, reason.clone()).await {
        Ok(()) => reason,
        Err(error) => format!("{reason}; recording the failure also failed: {error}"),
    };
    ReviewResult::JobFailed { reason }
}

pub(super) async fn reuse(store: &Store, job: &EnqueuedJob, run: RunId) -> ReviewResult {
    if let Err(error) = store.finish_reused(job.job_id, run).await {
        return job_failed(store, job, error.to_string()).await;
    }
    crash_point!("stored");
    match store.stored_result(job.job_id).await {
        Ok(stored) => ReviewResult::from_stored(stored, true),
        Err(error) => ReviewResult::JobFailed {
            reason: error.to_string(),
        },
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum JobEnd {
    Failed,
    Requeued {
        retry: Option<(u32, u32)>,
        next_attempt_at: Option<i64>,
    },
}

impl From<&crate::recovery::rules::JobNext> for JobEnd {
    fn from(next: &crate::recovery::rules::JobNext) -> Self {
        use crate::recovery::rules::JobNext;
        match next {
            JobNext::Fail { .. } => Self::Failed,
            JobNext::Requeue {
                retry_count,
                max_retries,
                next_attempt_at,
            } => Self::Requeued {
                retry: next_attempt_at.and_then(|_| max_retries.map(|max| (*retry_count, max))),
                next_attempt_at: *next_attempt_at,
            },
        }
    }
}
