use super::{EnqueueRequest, Enqueued, EnqueuedJob};
use crate::app::ProcessInfo;
use crate::error::Error;
use crate::pr_url::PrRef;
use crate::store::{Store, now_ms};
use rusqlite::{TransactionBehavior, params};

impl Store {
    pub async fn enqueue_review(
        &self,
        pr: &PrRef,
        process: &ProcessInfo,
        request: EnqueueRequest,
    ) -> Result<Enqueued, Error> {
        let (owner, repo, number) = (pr.owner.clone(), pr.repo.clone(), pr.number);
        let (pid, started) = (process.pid, process.started_at_ms);
        self.call(move |conn| {
            let number = i64::try_from(number)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let now = now_ms();
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tx.execute(
                "INSERT INTO targets (owner, repo, number, created_at) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (owner, repo, number) DO NOTHING",
                params![owner, repo, number, now],
            )?;
            let target_id = tx.query_row(
                "SELECT id FROM targets WHERE owner = ?1 AND repo = ?2 AND number = ?3",
                params![owner, repo, number],
                |row| row.get(0),
            )?;
            tx.execute(
                "INSERT INTO jobs (kind, target_id, state, enqueued_by_pid,
                 enqueued_by_started_at, created_at, updated_at, origin, force)
                 VALUES ('review', ?1, 'queued', ?2, ?3, ?4, ?4, ?5, ?6)",
                params![
                    target_id,
                    pid,
                    started,
                    now,
                    request.origin.as_str(),
                    request.force
                ],
            )?;
            let job_id = tx.last_insert_rowid();
            tx.commit()?;
            Ok(Enqueued {
                job: EnqueuedJob { target_id, job_id },
                attached: false,
            })
        })
        .await
    }
}
