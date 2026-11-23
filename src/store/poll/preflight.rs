use super::schedule;
use crate::error::Error;
use crate::store::{Store, jobs::EnqueuedJob, now_ms};
use rusqlite::{TransactionBehavior, params};
use std::time::Duration;

pub(crate) enum Admission {
    Ready,
    Draft,
    NewHead {
        head: String,
        debounce: Duration,
        force: bool,
    },
}

impl Store {
    pub async fn admit_automatic(
        &self,
        job: &EnqueuedJob,
        admission: Admission,
    ) -> Result<bool, Error> {
        let (job, target) = (job.job_id, job.target_id);
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let now = now_ms();
            let stopped = tx.execute(
                "UPDATE jobs SET state = 'cancelled', updated_at = ?2, next_attempt_at = NULL
                 WHERE id = ?1 AND failure_reason IS NOT NULL AND state = 'running'",
                params![job, now],
            )? != 0;
            let admitted = if stopped {
                false
            } else if matches!(admission, Admission::Draft) {
                tx.execute("UPDATE jobs SET state = 'queued', held = 1, updated_at = ?2 WHERE id = ?1", params![job, now])?;
                false
            } else if let Admission::NewHead { head, debounce, force } = admission {
                let delay = i64::try_from(debounce.as_millis()).unwrap_or(i64::MAX);
                tx.execute(
                    "UPDATE targets SET head_sha = ?2, head_checked_at = ?3, queued_head = ?2 WHERE id = ?1",
                    params![target, head, now],
                )?;
                if !force && schedule::attempted(&tx, target, &head)?
                    && !tx.query_row(
                        "SELECT EXISTS(SELECT 1 FROM runs WHERE target_id = ?1 AND head_sha = ?2 AND state = 'succeeded')",
                        params![target, head], |row| row.get::<_, bool>(0),
                    )?
                {
                    tx.execute("UPDATE jobs SET state = 'cancelled', failure_reason = 'new head already attempted', next_attempt_at = NULL, updated_at = ?2 WHERE id = ?1", params![job, now])?;
                } else {
                    schedule::retarget(&tx, job, &head, now, now.saturating_add(delay))?;
                }
                false
            } else {
                true
            };
            tx.commit()?;
            Ok(admitted)
        }).await
    }
}
