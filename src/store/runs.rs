use super::jobs::EnqueuedJob;
use super::records::{Metrics, Revision};
use super::retry::apply_job_next;
use super::{JobId, RunId, Store, now_ms};
use crate::agent::{FailureDetail, FailureKind};
use crate::cancel::StopReason;
use crate::error::Error;
use crate::recovery::rules::JobNext;
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
        failure: (FailureKind, Option<StopReason>),
        detail: FailureDetail,
        metrics: Metrics,
        next: &JobNext,
    ) -> Result<(), Error> {
        let (kind, stop) = failure;
        let next = next.clone();
        let detail = serde_json::to_string(&detail)?;
        let cause = stop.filter(|reason| {
            matches!(kind, FailureKind::Interrupted)
                && matches!(reason, StopReason::Shutdown | StopReason::UserCancel)
        });
        self.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let now = now_ms();
            tx.execute(
                "UPDATE runs SET state = 'failed', failure_kind = ?1, failure_detail = ?2,
                 total_cost_usd = ?3, duration_ms = ?4, finished_at = ?5, interrupted_cause = ?7
                 WHERE id = ?6",
                params![
                    kind.as_str(),
                    detail,
                    metrics.total_cost_usd,
                    metrics.duration_ms,
                    now,
                    run,
                    cause.map(StopReason::as_str)
                ],
            )?;
            apply_job_next(&tx, job, &next, now)?;
            tx.commit()
        })
        .await
    }

    pub async fn set_agent_process(
        &self,
        run: RunId,
        process: u32,
        group: u32,
    ) -> Result<(), Error> {
        self.call(move |conn| {
            let sql = "UPDATE runs SET agent_pid = ?1, agent_pgid = ?2 WHERE id = ?3";
            conn.execute(sql, params![process, group, run]).map(drop)
        })
        .await
    }
}
