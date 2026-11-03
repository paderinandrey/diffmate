use super::QueuedJob;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::store::jobs::EnqueuedJob;
use crate::store::{Store, now_ms};
use rusqlite::OptionalExtension;

impl Store {
    #[cfg_attr(not(test), expect(dead_code))]
    pub async fn next_runnable_job(&self, now: i64) -> Result<Option<QueuedJob>, Error> {
        self.call(move |conn| {
            conn.query_row(
                "SELECT j.id, j.target_id, t.owner, t.repo, t.number FROM jobs j
             JOIN targets t ON t.id = j.target_id WHERE j.kind = 'review' AND j.state = 'queued'
             AND (j.next_attempt_at IS NULL OR j.next_attempt_at <= ?1) ORDER BY j.id LIMIT 1",
                [now],
                |row| {
                    Ok(QueuedJob {
                        job: EnqueuedJob {
                            job_id: row.get(0)?,
                            target_id: row.get(1)?,
                        },
                        pr: PrRef {
                            owner: row.get(2)?,
                            repo: row.get(3)?,
                            number: u64::try_from(row.get::<_, i64>(4)?).map_err(|error| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    4,
                                    rusqlite::types::Type::Integer,
                                    Box::new(error),
                                )
                            })?,
                        },
                    })
                },
            )
            .optional()
        })
        .await
    }

    #[cfg_attr(not(test), expect(dead_code))]
    pub async fn earliest_next_attempt(&self) -> Result<Option<i64>, Error> {
        self.call(|conn| {
            conn.query_row(
                "SELECT min(next_attempt_at) FROM jobs WHERE kind = 'review'
             AND state = 'queued' AND next_attempt_at > ?1",
                [now_ms()],
                |row| row.get(0),
            )
        })
        .await
    }
}
