use super::records::JobOrigin;
use super::{JobId, Store};
use crate::error::Error;
use crate::recovery::rules::{JobNext, RetryState};
use rusqlite::{Transaction, params};

mod queue;
mod revision;
#[cfg(test)]
mod tests;

pub(crate) struct QueuedJob {
    pub job: super::jobs::EnqueuedJob,
    pub pr: crate::pr_url::PrRef,
    pub origin: JobOrigin,
}

pub(super) fn apply_job_next(
    tx: &Transaction,
    job: JobId,
    mut next: JobNext,
    now: i64,
) -> rusqlite::Result<JobNext> {
    if matches!(next, JobNext::Requeue { .. }) {
        let reason: Option<String> = tx.query_row(
            "SELECT failure_reason FROM jobs WHERE id = ?1",
            [job],
            |row| row.get(0),
        )?;
        if let Some(reason) = reason {
            next = JobNext::Fail {
                reason,
                max_retries: None,
            };
        }
    }
    match &next {
        JobNext::Requeue {
            retry_count,
            max_retries,
            next_attempt_at,
        } => {
            tx.execute(
                "UPDATE jobs SET state = 'queued', retry_count = ?1, max_retries = ?2,
                 next_attempt_at = ?3, stop_reason = NULL, failure_reason = NULL, updated_at = ?4 WHERE id = ?5",
                params![retry_count, max_retries, next_attempt_at, now, job],
            )?;
        }
        JobNext::Fail {
            reason,
            max_retries,
        } => {
            tx.execute(
                "UPDATE jobs SET state = 'failed', failure_reason = ?1,
                 max_retries = COALESCE(?2, max_retries), next_attempt_at = NULL, updated_at = ?3 WHERE id = ?4",
                params![reason, max_retries, now, job],
            )?;
        }
    }
    Ok(next)
}

impl Store {
    pub async fn job_retry_state(&self, job: JobId) -> Result<(JobOrigin, RetryState), Error> {
        self.call(move |conn| {
            conn.query_row(
                "SELECT origin, retry_count, max_retries FROM jobs WHERE id = ?1",
                [job],
                |row| {
                    Ok((
                        JobOrigin::parse(&row.get::<_, String>(0)?)?,
                        RetryState {
                            retry_count: row.get(1)?,
                            max_retries: row.get(2)?,
                        },
                    ))
                },
            )
        })
        .await
    }

    pub async fn resolve_job(&self, job: JobId, next: JobNext) -> Result<JobNext, Error> {
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let next = apply_job_next(&tx, job, next, super::now_ms())?;
            tx.commit()?;
            Ok(next)
        })
        .await
    }
}
