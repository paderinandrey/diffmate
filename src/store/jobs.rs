use super::{JobId, Store, TargetId, now_ms};
use crate::app::ProcessInfo;
use crate::error::Error;
use crate::pr_url::PrRef;
use rusqlite::{TransactionBehavior, params};

pub(crate) struct EnqueuedJob {
    pub target_id: TargetId,
    pub job_id: JobId,
}

impl Store {
    pub async fn enqueue_review(
        &self,
        pr: &PrRef,
        process: &ProcessInfo,
    ) -> Result<EnqueuedJob, Error> {
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
                 enqueued_by_started_at, created_at, updated_at)
                 VALUES ('review', ?1, 'queued', ?2, ?3, ?4, ?4)",
                params![target_id, pid, started, now],
            )?;
            let job_id = tx.last_insert_rowid();
            tx.commit()?;
            Ok(EnqueuedJob { target_id, job_id })
        })
        .await
    }

    pub async fn start_job(&self, job: JobId) -> Result<(), Error> {
        self.call(move |conn| {
            let sql = "UPDATE jobs SET state = 'running', updated_at = ?1 WHERE id = ?2";
            conn.execute(sql, params![now_ms(), job]).map(drop)
        })
        .await
    }

    pub async fn fail_job(&self, job: JobId, reason: String) -> Result<(), Error> {
        self.call(move |conn| {
            let sql = "UPDATE jobs SET state = 'failed', failure_reason = ?1, updated_at = ?2
                       WHERE id = ?3";
            conn.execute(sql, params![reason, now_ms(), job]).map(drop)
        })
        .await
    }
}
