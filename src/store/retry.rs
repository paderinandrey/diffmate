use super::records::JobOrigin;
use super::{JobId, Store};
use crate::error::Error;
use crate::recovery::rules::{JobNext, RetryState};
use rusqlite::{Transaction, params};

pub(super) fn apply_job_next(
    tx: &Transaction,
    job: JobId,
    next: &JobNext,
    now: i64,
) -> rusqlite::Result<()> {
    match next {
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
    Ok(())
}

impl Store {
    #[expect(dead_code)]
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
}
