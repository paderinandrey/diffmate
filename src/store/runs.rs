use super::jobs::EnqueuedJob;
use super::records::{Metrics, Revision};
use super::{JobId, RunId, Store, now_ms};
use crate::agent::{FailureDetail, FailureKind};
use crate::error::Error;
use rusqlite::{TransactionBehavior, params};

impl Store {
    pub async fn insert_running_run(
        &self,
        job: &EnqueuedJob,
        revision: &Revision,
    ) -> Result<RunId, Error> {
        let (target, job_id) = (job.target_id, job.job_id);
        let (head, merge_base) = (revision.head_sha.clone(), revision.merge_base_sha.clone());
        let (tip, base_ref) = (revision.base_tip_sha.clone(), revision.base_ref.clone());
        self.call(move |conn| {
            let sql = "INSERT INTO runs (target_id, job_id, state, head_sha, merge_base_sha,
                       base_tip_sha, base_ref, created_at) VALUES (?1, ?2, 'running', ?3, ?4, ?5, ?6, ?7)";
            conn.execute(sql, params![target, job_id, head, merge_base, tip, base_ref, now_ms()])?;
            Ok(conn.last_insert_rowid())
        })
        .await
    }

    pub async fn finish_failed(
        &self,
        run: RunId,
        job: JobId,
        kind: FailureKind,
        detail: FailureDetail,
        metrics: Metrics,
    ) -> Result<(), Error> {
        let detail = serde_json::to_string(&detail)?;
        let reason = format!("run {run} failed ({})", kind.as_str());
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let now = now_ms();
            tx.execute(
                "UPDATE runs SET state = 'failed', failure_kind = ?1, failure_detail = ?2,
                 total_cost_usd = ?3, duration_ms = ?4, finished_at = ?5 WHERE id = ?6",
                params![
                    kind.as_str(),
                    detail,
                    metrics.total_cost_usd,
                    metrics.duration_ms,
                    now,
                    run
                ],
            )?;
            let sql = "UPDATE jobs SET state = 'failed', failure_reason = ?1, updated_at = ?2
                       WHERE id = ?3";
            tx.execute(sql, params![reason, now, job])?;
            tx.commit()
        })
        .await
    }
}
